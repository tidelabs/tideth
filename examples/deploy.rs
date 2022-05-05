// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of tideth.

// tideth is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with tideth.  If not, see <http://www.gnu.org/licenses/>.

use dotenv::dotenv;
use ethcontract::{
  transport::DynTransport, web3::types::TransactionRequest, Account, Address, Http, PrivateKey,
  Web3, H160, U256,
};
use futures::{channel::mpsc, stream::StreamExt};
use std::str::FromStr;
use tideth::{
  error::Error,
  router::{DepositEvent, Receiver, RouterClient, Sender},
  safe::SafeClient,
};

// ethcontract::contract!("contracts/RustCoin.sol/RustCoin.json");

// const URL: &str = "https://ropsten.infura.io/v3/2155bb47548546a592f44b2786b59590";
// const URL: &str = "http://localhost:8545";

// empty to deploy anew
const ROUTER_ADDRESS: Option<&str> = None;
const SAFE_ADDRESS: Option<&str> = None;

#[tokio::main]
async fn main() {
  dotenv().ok();

  // chain id
  let chain_id: u64 = std::env::var("CHAIN_ID")
    .unwrap_or("1337".to_string())
    .parse::<u64>()
    .unwrap_or(1337);

  // private key from metamask
  let priv_key_string = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
  let priv_key = PrivateKey::from_str(priv_key_string.as_str()).expect("couldnt parse private key");

  let eth_url: String = std::env::var("ETH_URL").unwrap_or("http://localhost:8545".to_string());
  let web3 = Web3::new(DynTransport::new(
    Http::new(eth_url.as_str()).expect("couldnt setup web3"),
  ));

  // either imported account from metamask
  let my_account = Account::Offline(priv_key, Some(chain_id));
  let my_address = my_account.address();
  println!("my address {:?}", my_address);

  // check my balance
  let my_eth_balance = web3
    .eth()
    .balance(my_address, None)
    .await
    .expect("couldnt get ETH balance");

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");

  // if its zero, send some ETH for gas
  if accounts.len() > 0 && my_eth_balance.as_u128() == 0 {
    // but only if there is a funded account
    let tx_object = TransactionRequest {
      from: accounts[0],
      to: Some(my_address),
      value: Some(U256::exp10(17)), //0.1 eth
      ..Default::default()
    };
    match web3.eth().send_transaction(tx_object).await {
      Ok(_) => println!("ETH TX succeeded!"),
      Err(e) => println!("ETH TX FAILED {}", e),
    };
  }

  // owners vec
  let owners_string = std::env::var("OWNERS").unwrap();
  let owners_iter = owners_string.as_str().split(",");
  let mut safe_owners: Vec<H160> = owners_iter
    .map(|a| Address::from_str(a).expect("couldnt parse H160"))
    .collect();
  if safe_owners.len() < 2 {
    panic!("not enough owners");
  }
  // the leader is first in the (pre-sorted) list
  let leader_address = safe_owners[0];
  // sort them now
  safe_owners.sort();
  println!("THE OWNERS {:?}", safe_owners);

  // threshold
  let threshold: u64 = std::env::var("THRESHOLD")
    .unwrap_or("2".to_string())
    .parse::<u64>()
    .unwrap_or(2);

  // deploy router
  let mut router = RouterClient::new(&web3, ROUTER_ADDRESS).expect("derp");
  if let None = ROUTER_ADDRESS {
    router
      .deploy(my_account.clone())
      .await
      .expect("Didnt deploy");
  }
  println!("DEPLOYED ROUTER!");

  // Deploy an ERC20
  // let erc20 = RustCoin::builder(&web3)
  //   //.gas(1_000_000_u64.into())
  //   .from(my_account.clone())
  //   .deploy()
  //   .await
  //   .expect("Couldn't deploy the ERC20");

  // add the ERC20 to the router acceptlist
  // router
  //   .add_token(my_account.clone(), erc20.address())
  //   .await
  //   .expect("couldnt add token");

  let added_usdt = add_token_from_env("USDT", &router, &my_account).await;
  println!("ADDED USDT {:?}", added_usdt);

  let added_usdc = add_token_from_env("USDC", &router, &my_account).await;
  println!("ADDED USDC {:?}", added_usdc);

  // deploy safe, and set up with owners and threshold
  let mut safe = SafeClient::new(&web3, SAFE_ADDRESS).expect("derp");
  if let None = SAFE_ADDRESS {
    safe.deploy(my_account.clone()).await.expect("Didnt deploy");
    safe
      .setup(my_account.clone(), safe_owners.clone(), threshold)
      .await
      .expect("Couldnt setup");
  }
  let pending_owner = router
    .pending_owner()
    .await
    .expect("couldnt get currenct owner");

  // transfer router to safe
  if pending_owner != safe.address() {
    router
      .transfer_ownership(my_account.clone(), safe.address())
      .await
      .expect("Couldnt transfer router ownership to safe");
    // router
    //   .claim_ownership(safe.address())
    //   .await
    //   .expect("Couldnt transfer router ownership to safe");
    println!("safe ownding owner of router!");
  } else {
    println!("safe already owns router");
  }

  let pending_owner2 = router.pending_owner().await.expect("couldnt call owner");
  assert_eq!(pending_owner2, safe.address(), "safe should own router");

  println!("leader addy {:?}", leader_address);
  // check safe balance
  let leader_eth_balance = web3
    .eth()
    .balance(leader_address, None)
    .await
    .expect("couldnt get ETH balance");

  // if its zero, send some ETH for gas
  if accounts.len() > 0 && leader_eth_balance.as_u128() == 0 {
    // but only if there is a funded account
    let tx_object = TransactionRequest {
      from: accounts[0],
      to: Some(leader_address),
      value: Some(U256::exp10(16)), //0.1 eth
      ..Default::default()
    };
    match web3.eth().send_transaction(tx_object).await {
      Ok(_) => println!("ETH TX to LEADER succeeded!"),
      Err(e) => println!("ETH TX FAILED {}", e),
    };
  }

  let safe_eth_balance = web3
    .eth()
    .balance(safe.address(), None)
    .await
    .expect("couldnt get ETH balance");

  // if its zero, send some ETH for gas
  if accounts.len() > 0 && safe_eth_balance.as_u128() == 0 {
    // but only if there is a funded account
    let tx_object = TransactionRequest {
      from: accounts[0],
      to: Some(safe.address()),
      value: Some(U256::exp10(17)), //0.1 eth
      ..Default::default()
    };
    match web3.eth().send_transaction(tx_object).await {
      Ok(_) => println!("ETH TX to SAFE succeeded!"),
      Err(e) => println!("ETH TX FAILED {}", e),
    };
  }

  // println!("===============");
  // println!("RustCoin erc20 address {:?}", erc20.address());
  println!("===============");
  println!("Router address {:?}", router.address());
  println!("===============");
  println!("Safe address {:?}", safe.address());
  println!("===============");
  println!("http://localhost:8081?router={:?}&account=0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c", router.address());
  println!("===============");

  let (eth_deposit_tx, mut eth_deposit_rx): (Sender<DepositEvent>, Receiver<DepositEvent>) =
    mpsc::unbounded();

  tokio::task::spawn(async move {
    router.subscribe_events(eth_deposit_tx).await;
  });

  if let Some(deposit) = eth_deposit_rx.next().await {
    println!(
      "Received a deposit event! to asset {:?} with amount {}",
      deposit.asset, deposit.amount
    );
  }
}

async fn add_token_from_env(
  symbol: &str,
  router: &RouterClient,
  my_account: &Account<DynTransport>,
) -> Result<String, Error> {
  let usdt = std::env::var(symbol).unwrap_or("".to_string());
  if usdt.len() == 42 {
    if let Ok(addy) = Address::from_str(usdt.as_str()) {
      router.add_token(my_account.clone(), addy).await?;
      return Ok(addy.to_string());
    }
  }
  Ok("".to_string())
}
