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

use ethcontract::{
  transport::DynTransport,
  web3::types::{TransactionRequest, U256},
  Account, Bytes, Http, Web3, H160,
};
use iota_stronghold::{Location, ProcResult, Procedure, ResultMessage, Stronghold};
use tideth::{router::RouterClient, safe::SafeClient};

ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");

const ROUTER_ADDRESS: Option<&str> = None;
const SAFE_ADDRESS: Option<&str> = None;

#[tokio::test]
async fn main() {
  let mut nonce = 0;
  let chain_id = 1337;
  let web3 = Web3::new(DynTransport::new(
    Http::new("http://localhost:8545").expect("couldnt setup web3"),
  ));

  let (strong1, loc1, account1) = init_account(&web3, chain_id, 1).await;
  let address1 = account1.address();
  let (strong2, loc2, account2) = init_account(&web3, chain_id, 2).await;
  let address2 = account2.address();
  let (_strong3, _loc3, account3) = init_account(&web3, chain_id, 3).await;
  let address3 = account3.address();
  //
  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  // prefunded account
  let zero_account = Account::Local(accounts[0], None);

  // ETH tx
  let tx_object = TransactionRequest {
    from: accounts[0],
    to: Some(address1),
    value: Some(U256::exp10(18)), // 1 eth
    ..Default::default()
  };

  // Send the tx to quorum leader
  match web3.eth().send_transaction(tx_object).await {
    Ok(_) => println!("ETH tx succeeded!"),
    Err(e) => println!("TX FAILED {}", e),
  };

  let one_balance = web3
    .eth()
    .balance(address1, None)
    .await
    .expect("couldnt get balance");
  println!("one balance {:?}", one_balance);

  let mut router = RouterClient::new(&web3, ROUTER_ADDRESS).expect("derp");
  if let None = ROUTER_ADDRESS {
    router
      .deploy(zero_account.clone())
      .await
      .expect("Didnt deploy");
  }

  // Deploy an ERC20 and send 100 tokens to the router
  let erc20 = RustCoin::builder(&web3)
    //.gas(1_000_000_u64.into())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  let coin_amt = 100;
  let send_amt = 100;
  erc20
    .transfer(router.address(), coin_amt.into())
    .send()
    .await
    .expect("Couldn't send the ERC20 to the router");
  assert_eq!(
    erc20
      .balance_of(router.address())
      .call()
      .await
      .expect("Couldn't get router's balance"),
    coin_amt.into()
  );

  println!(
    "Router RustCoin balance = {:?}! {:?}",
    coin_amt,
    router.address()
  );

  let mut safe = SafeClient::new(&web3, SAFE_ADDRESS).expect("derp");
  if let None = SAFE_ADDRESS {
    safe
      .deploy(zero_account.clone())
      .await
      .expect("Didnt deploy");
    let owners = vec![address1, address2, address3];
    safe
      .setup(zero_account.clone(), owners, 2)
      .await
      .expect("Couldnt setup");
  }
  let current_router_owner = router.owner().await.expect("couldnt get currenct owner");

  if current_router_owner != safe.address() {
    router
      .transfer_ownership(zero_account.clone(), safe.address())
      .await
      .expect("Couldnt transfer router ownership to safe");
    let claimdata = router
      .claim_ownership_data()
      .expect("could make claim_ownership_data");
    let tx_data = safe
      .encode_data(router.address(), 0, claimdata.clone(), nonce)
      .await
      .expect("couldnt encode claim data");

    let signatures = signs(
      tx_data,
      strong1.clone(),
      loc1.clone(),
      address1,
      strong2.clone(),
      loc2.clone(),
      address2,
    )
    .await;
    safe
      .exec_with_data(
        zero_account.clone(),
        router.address(),
        claimdata,
        signatures,
        0,
      )
      .await
      .expect("Couldn't execute the Claim");
    // increment the nonce
    nonce = nonce + 1;
    println!("safe owns router!");
  } else {
    println!("safe already owns router");
  }

  // ETH tx to SAFE
  let eth_tx_object = TransactionRequest {
    from: accounts[0],
    to: Some(safe.address()),
    value: Some(U256::exp10(17)), //0.1 eth
    ..Default::default()
  };

  // Send the tx to localhost
  match web3.eth().send_transaction(eth_tx_object).await {
    Ok(_) => println!("ETH tx succeeded!"),
    Err(e) => println!("TX FAILED {}", e),
  };

  let safe_eth_balance = web3
    .eth()
    .balance(safe.address(), None)
    .await
    .expect("couldnt get SAFE balance");
  assert_eq!(safe_eth_balance, U256::exp10(17));

  println!("SAFE ETH balance {}", safe_eth_balance);

  // send RustCoin to accounts[3]
  let withdrawaldata = router
    .erc20_withdrawal_data(accounts[3], erc20.address(), send_amt)
    .expect("Couldnt build withdrawal data");

  // must increment nonce by one each EXEC
  let wtxdata = safe
    .encode_data(router.address(), 0, withdrawaldata.clone(), nonce)
    .await
    .expect("couldnt encode withdrawal data");

  nonce = nonce + 1;

  let signatures = signs(
    wtxdata,
    strong1.clone(),
    loc1.clone(),
    address1,
    strong2.clone(),
    loc2.clone(),
    address2,
  )
  .await;

  safe
    .exec_with_data(
      account1.clone(),
      router.address(),
      withdrawaldata,
      signatures,
      0,
    )
    .await
    .expect("Couldn't execute the TX");

  println!("exectued withdrawal!");
  assert_eq!(
    erc20
      .balance_of(router.address())
      .call()
      .await
      .expect("Couldn't get router's balance"),
    (coin_amt - send_amt).into()
  );
  println!("router has {:?} erc20 left!", (coin_amt - send_amt));

  assert_eq!(
    erc20
      .balance_of(accounts[3])
      .call()
      .await
      .expect("Couldn't get accounts[3] balance"),
    send_amt.into()
  );
  println!("accounts[3] has {:?} erc20!", send_amt);
  //
  //
  // now withdraw ETH to accounts[3]
  let initial_accounts3_eth_balance = web3
    .eth()
    .balance(accounts[3], None)
    .await
    .expect("couldnt get accounts[3] balance");
  println!(
    "initial accounts[3] balance {}",
    initial_accounts3_eth_balance.as_u128()
  );
  let ethtxdata = safe
    .encode_eth_tx(accounts[3], send_amt, nonce)
    .await
    .expect("couldnt encode ETH withdrawal data");

  let eth_signed1 = sign(&ethtxdata.0, strong1, loc1).await;
  let eth_signed2 = sign(&ethtxdata.0, strong2, loc2).await;

  let sorted_sigs = if address1 < address2 {
    vec![eth_signed1, eth_signed2]
  } else {
    vec![eth_signed2, eth_signed1]
  };
  let mut eth_signatures = vec![];
  for mut sig in sorted_sigs {
    eth_signatures.append(&mut sig);
  }

  safe
    .exec_eth_tx(account1, accounts[3], send_amt, eth_signatures)
    .await
    .expect("Couldn't execute the ETH TX");
  println!("EXECUTED ETH WITHDRAWAL!");

  let accounts3_eth_balance = web3
    .eth()
    .balance(accounts[3], None)
    .await
    .expect("couldnt get accounts[3] balance");
  assert_eq!(
    accounts3_eth_balance.as_u128(),
    initial_accounts3_eth_balance.as_u128() + send_amt
  );

  let safe_eth_balance = web3
    .eth()
    .balance(safe.address(), None)
    .await
    .expect("couldnt get SAFE balance");
  assert_eq!(
    safe_eth_balance.as_u128(),
    U256::exp10(17).as_u128() - send_amt
  );
}

async fn signs(
  wtxdata: Bytes<Vec<u8>>,
  strong1: Stronghold,
  loc1: Location,
  address1: H160,
  strong2: Stronghold,
  loc2: Location,
  address2: H160,
) -> Vec<u8> {
  let signed1 = sign(&wtxdata.0, strong1.clone(), loc1.clone()).await;
  let signed2 = sign(&wtxdata.0, strong2.clone(), loc2.clone()).await;
  let sorted_sigs = if address1 < address2 {
    vec![signed1, signed2]
  } else {
    vec![signed2, signed1]
  };
  let mut signatures = vec![];
  for mut sig in sorted_sigs {
    signatures.append(&mut sig);
  }
  signatures
}

async fn sign(msg: &Vec<u8>, stronghold: Stronghold, location: Location) -> Vec<u8> {
  let hash = keccak256(msg.as_ref());
  match stronghold
    .runtime_exec(Procedure::Secp256k1Sign {
      private_key: location,
      msg: Box::new(hash),
    })
    .await
  {
    ProcResult::Secp256k1Sign(ResultMessage::Ok(res)) => {
      let (sig, recid) = res;
      let record_id = recid.as_u8() + 27;
      let mut sigvec = sig.to_bytes().to_vec();
      sigvec.push(record_id);
      sigvec
    }
    _ => panic!("cant sign"),
  }
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
  use tiny_keccak::{Hasher, Keccak};
  let mut output = [0u8; 32];
  let mut hasher = Keccak::v256();
  hasher.update(bytes);
  hasher.finalize(&mut output);
  output
}

async fn init_account(
  web3: &Web3<DynTransport>,
  chain_id: u64,
  keynum: u8,
) -> (Stronghold, Location, Account<DynTransport>) {
  let (tx, rx) = std::sync::mpsc::channel();
  std::thread::spawn(move || {
    let system = actix::System::new();
    let stronghold = system
      .block_on(Stronghold::init_stronghold_system(b"path".to_vec(), vec![]))
      .unwrap();
    tx.send(stronghold).unwrap();
    system.run().expect("actix system run failed");
  });
  let stronghold = rx.recv().unwrap();

  let keypair_location = Location::generic("SR25519", "keypair");

  match stronghold
    .runtime_exec(Procedure::Secp256k1Store {
      key: [keynum; 32].to_vec(),
      output: keypair_location.clone(),
      hint: [0u8; 24].into(),
    })
    .await
  {
    ProcResult::Secp256k1Generate(ResultMessage::OK) => (),
    r => panic!("unexpected result: {:?}", r),
  }

  let accounts = web3.accounts();
  (
    stronghold.clone(),
    keypair_location.clone(),
    Account::Stronghold(stronghold, accounts, keypair_location, Some(chain_id)),
  )
}
