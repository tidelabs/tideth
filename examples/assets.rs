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
use ethcontract::{transport::DynTransport, Account, Http, PrivateKey, Web3};
use std::str::FromStr;

ethcontract::contract!("artifacts/contracts/Tether.sol/Tether.json");
ethcontract::contract!("artifacts/contracts/Circle.sol/Circle.json");

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

  // Deploy an ERC20
  let usdt = Tether::builder(&web3)
    //.gas(1_000_000_u64.into())
    .from(my_account.clone())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  // Deploy an ERC20
  let usdc = Circle::builder(&web3)
    //.gas(1_000_000_u64.into())
    .from(my_account.clone())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  println!("===============");
  println!("\"usdt_address\": {:?}", usdt.address());
  println!("===============");
  println!("\"usdc_address\": {:?}", usdc.address());
}
