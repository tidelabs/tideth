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

use ethcontract::{Address, H160};
use std::str::FromStr;
use tideth::{config, safe::SafeClient};

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, my_account, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  let safe_address = conf.safe_address.expect("no safe address");
  let mut safe = SafeClient::new(&web3, Some(safe_address.as_str())).expect("derp2");

  let mut safe_owners: Vec<H160> = conf
    .owners
    .iter()
    .map(|a| Address::from_str(a).expect("couldnt parse H160"))
    .collect();
  if safe_owners.len() < conf.threshold as usize {
    panic!("not enough owners");
  }
  // sort them now
  safe_owners.sort();
  println!("OWNERS {:?}", safe_owners);

  safe
    .setup(my_account.clone(), safe_owners.clone(), conf.threshold)
    .await
    .expect("Couldnt setup");

  println!("=> safe is setup!");
}
