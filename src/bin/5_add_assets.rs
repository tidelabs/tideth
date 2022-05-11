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

use ethcontract::Address;
use std::str::FromStr;
use tideth::config;
use tideth::router::RouterClient;

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, my_account, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  let router_address = conf.router_address.expect("no router address");
  let router = RouterClient::new(&web3, Some(router_address.as_str())).expect("derp");

  if let Some(addy) = conf.usdt_address {
    let address = Address::from_str(addy.as_str()).expect("failed to parse USDT address");
    let accepted = router
      .is_accepted(address.clone())
      .await
      .expect("couldnt call is_accepted");
    if accepted {
      println!("usdt already accepted");
    } else {
      router
        .add_token(my_account.clone(), address)
        .await
        .expect("could not add USDT");
      println!("added USDT!");
    }
  } else {
    println!("no USDT address");
  }

  if let Some(addy) = conf.usdc_address {
    let address = Address::from_str(addy.as_str()).expect("failed to parse USDC address");
    let accepted = router
      .is_accepted(address.clone())
      .await
      .expect("couldnt call is_accepted");
    if accepted {
      println!("USDC already accepted");
    } else {
      router
        .add_token(my_account.clone(), address)
        .await
        .expect("could not add USDC");
      println!("added USDC!");
    }
  } else {
    println!("no USDC address");
  }
}
