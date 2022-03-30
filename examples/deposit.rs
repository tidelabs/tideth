// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of tidext.

// tidext is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with tidext.  If not, see <http://www.gnu.org/licenses/>.

use dotenv::dotenv;
use ethcontract::{transport::DynTransport, Account, Address, Http, PrivateKey, Web3, U256};
use std::str::FromStr;
use tideth::router::RouterClient;

ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");

// const URL: &str = "https://ropsten.infura.io/v3/2155bb47548546a592f44b2786b59590";
// const URL: &str = "http://localhost:8545";

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
  let my_account: Account<DynTransport> = Account::Offline(priv_key, Some(chain_id));
  let my_address = my_account.address();
  println!("my address {:?}", my_address);

  // check my balance
  let _my_eth_balance = web3
    .eth()
    .balance(my_address, None)
    .await
    .expect("couldnt get ETH balance");

  let _accounts = web3.eth().accounts().await.expect("getAccounts failed");

  let router_address_string = std::env::var("ROUTER_ADDRESS")
    .unwrap_or("0xdbd4910f54a3751f964cb3bad99374134b2e34e7".to_string());
  let router_address = Some(router_address_string.as_str());

  let router = RouterClient::new(&web3, router_address).expect("derp");

  let test_account_string = std::env::var("TEST_ACCOUNT").unwrap_or("".to_string());
  let test_account = hex::decode(test_account_string).expect("could hex decode test account");

  let asset_address_string = std::env::var("ASSET_ADDRESS").unwrap_or("".to_string());
  let asset_address =
    Address::from_str(asset_address_string.as_str()).expect("couldnt parse asset address");

  // deposit 0.01 tether
  router
    .deposit(
      my_account.clone(),
      test_account,
      asset_address,
      U256::exp10(16),
      None,
    )
    .await
    .expect("couldnt deposit");
}
