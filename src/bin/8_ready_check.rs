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

use ethcontract::H160;
use std::str::FromStr;
use tideth::config;
use tideth::router::RouterClient;
use tideth::safe::SafeClient;

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, _, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  let router_address = conf.router_address.expect("no router address");
  let router = RouterClient::new(&web3, Some(router_address.as_str())).expect("derp");

  let safe_address = conf.safe_address.expect("no safe address");
  let safe = SafeClient::new(&web3, Some(safe_address.as_str())).expect("derp2");

  let usdt_address_string = conf.usdt_address.expect("no usdt address");
  let usdt_address =
    H160::from_str(usdt_address_string.as_str()).expect("couldnt parse usdt address");
  let usdt_accepted = router
    .is_accepted(usdt_address)
    .await
    .expect("couldnt call accepted");
  assert!(usdt_accepted, "usdt is not accepted");

  let usdc_address_string = conf.usdc_address.expect("no usdt address");
  let usdc_address =
    H160::from_str(usdc_address_string.as_str()).expect("couldnt parse usdc address");
  let usdc_accepted = router
    .is_accepted(usdc_address)
    .await
    .expect("couldnt call accepted");
  assert!(usdc_accepted);

  let pending_owner = router.pending_owner().await.expect("couldnt call owner");
  assert_eq!(pending_owner, safe.address(), "safe should own router");
  println!("Safe is pending owner of Router");
}
