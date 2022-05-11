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

use tideth::config;
use tideth::safe::SafeClient;

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, my_account, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  if let Some(_) = conf.safe_factory_address {
    panic!("already a SAFE factory address");
  }
  let mut safe = SafeClient::new(&web3, None).expect("derp");
  let addy = safe
    .deploy_factory(my_account.clone())
    .await
    .expect("couldnt deply singleton");

  println!("Gnosis Safe factory address: {:?}", addy);
}
