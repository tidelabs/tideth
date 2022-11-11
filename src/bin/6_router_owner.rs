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

use tideth::{config, router::RouterClient, safe::SafeClient};

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, my_account, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  let router_address = conf.router_address.expect("no router address");
  let mut router = RouterClient::new(&web3, Some(router_address.as_str())).expect("derp");

  let safe_address = conf.safe_address.expect("no safe address");
  let safe = SafeClient::new(&web3, Some(safe_address.as_str())).expect("derp2");

  let pending_owner = router
    .pending_owner()
    .await
    .expect("couldnt get currenct owner");

  // transfer router to safe
  if pending_owner != safe.address() {
    println!("pending owner is not SAFE, transfer now");
    router
      .transfer_ownership(my_account.clone(), safe.address())
      .await
      .expect("Couldnt transfer router ownership to safe");
    println!("safe ownding owner of router!");
  } else {
    println!("safe already owns router");
  }

  let pending_owner2 = router.pending_owner().await.expect("couldnt call owner");
  assert_eq!(pending_owner2, safe.address(), "safe should own router");

  println!("=> pending owner is SAFE!");
}
