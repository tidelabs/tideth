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
use ethcontract::{transport::DynTransport, Account, Address, Http, PrivateKey, Web3, H160};
use std::str::FromStr;
use tideth::safe::SafeClient;

ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");

// const URL: &str = "https://ropsten.infura.io/v3/2155bb47548546a592f44b2786b59590";
// const URL: &str = "http://localhost:8545";

// empty to deploy anew
// const ROUTER_ADDRESS: Option<&str> = None;
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
  let _ = web3
    .eth()
    .balance(my_address, None)
    .await
    .expect("couldnt get ETH balance");

  let _accounts = web3.eth().accounts().await.expect("getAccounts failed");

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
  let _leader_address = safe_owners[0];
  // sort them now
  safe_owners.sort();

  // threshold
  let threshold: u64 = std::env::var("THRESHOLD")
    .unwrap_or("2".to_string())
    .parse::<u64>()
    .unwrap_or(2);

  // deploy safe, and set up with owners and threshold
  let mut safe = SafeClient::new(&web3, SAFE_ADDRESS).expect("derp");
  if let None = SAFE_ADDRESS {
    safe.deploy(my_account.clone()).await.expect("Didnt deploy");
    safe
      .setup(my_account, safe_owners.clone(), threshold)
      .await
      .expect("Couldnt setup");
  }
  println!("SAFE ADDY {:?}", safe.address());
}
