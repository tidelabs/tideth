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
use ethcontract::H256;
use std::convert::TryInto;
use tideth::{config, router::RouterClient};

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, _, conf) = config::init_web3(net.as_str(), false)
    .await
    .expect("failed to init web3");

  let router_address = conf.router_address.expect("no router address");
  let router = RouterClient::new(&web3, Some(router_address.as_str())).expect("derp");

  let deps = router
    .get_all_deposits(None)
    .await
    .expect("couldnt get all deps");
  //   println!("DEPS {:?}", deps);

  for dep in deps {
    let txid_slice: [u8; 32] = dep.tx_hash.try_into().expect("nope");
    let tx_hash: H256 = (&txid_slice).into();
    let tr = web3
      .eth()
      .transaction_receipt(tx_hash)
      .await
      .expect("couldnt get TX RECEIPT")
      .expect("NO TX RECIPET FOUND");
    println!("RECEIPT GAS USED {:?}", tr.gas_used);
  }
}
