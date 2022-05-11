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

use ethcontract::{transport::DynTransport, Account, Http, PrivateKey, Web3};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
  pub owners: Vec<String>,
  pub chain_id: u64,
  pub private_key: String,
  pub threshold: u64,
  pub eth_url: String,
  pub usdt_address: Option<String>,
  pub usdc_address: Option<String>,
  pub safe_address: Option<String>,
  pub safe_factory_address: Option<String>,
  pub router_address: Option<String>,
}

// utils for binaries
pub fn load_config(network: &str) -> std::result::Result<Config, Box<dyn std::error::Error>> {
  let fp = format!("config/config.{}.json", network);
  let mut settings = config::Config::default();
  let _ = settings.merge(config::File::with_name(fp.as_str()));
  let sets = settings
    .try_into::<Config>()
    .expect("could not load config");
  Ok(sets)
}

// utils for binaries
pub async fn init_web3(
  net: &str,
  check_balance: bool,
) -> std::result::Result<
  (Web3<DynTransport>, Account<DynTransport>, Config),
  Box<dyn std::error::Error>,
> {
  let conf = load_config(net).expect("could not load config");
  println!("{:?}", conf);
  let priv_key =
    PrivateKey::from_str(conf.private_key.as_str()).expect("couldnt parse private key");
  let web3 = Web3::new(DynTransport::new(
    Http::new(conf.eth_url.as_str()).expect("couldnt setup web3"),
  ));
  // either imported account from metamask
  let my_account = Account::Offline(priv_key, Some(conf.chain_id));

  if check_balance {
    let my_address = my_account.address();
    println!("my address {:?}", my_address);
    // check my balance
    let balance = web3
      .eth()
      .balance(my_address, None)
      .await
      .expect("couldnt get ETH balance");
    println!("my ETH balance {}", balance.as_u128());
    if balance.as_u128() == 0 {
      panic!("no ETH balance in the provided privkey (for deployment gas fees)");
    }
  }
  Ok((web3, my_account, conf))
}
