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

ethcontract::contract!("artifacts/contracts/Tether.sol/Tether.json");
ethcontract::contract!("artifacts/contracts/Circle.sol/Circle.json");

#[tokio::main]
async fn main() {
  let net = std::env::var("NETWORK").expect("NETWORK REQUIRED");
  let (web3, my_account, conf) = config::init_web3(net.as_str(), true)
    .await
    .expect("failed to init web3");

  let usdt_address: String = if let Some(addy) = conf.usdt_address {
    println!("USDT address already exists in conf");
    addy
  } else {
    let usdt_contract = Tether::builder(&web3)
      //.gas(1_000_000_u64.into())
      .from(my_account.clone())
      .deploy()
      .await
      .expect("Couldn't deploy USDT");
    format!("{:?}", usdt_contract.address())
  };

  let usdc_address: String = if let Some(addy) = conf.usdc_address {
    println!("USDC address already exists in conf");
    addy
  } else {
    let usdc_contract = Circle::builder(&web3)
      //.gas(1_000_000_u64.into())
      .from(my_account.clone())
      .deploy()
      .await
      .expect("Couldn't deploy USDC");
    format!("{:?}", usdc_contract.address())
  };

  println!("===============");
  println!("Tether address {:?}", usdt_address);
  println!("===============");
  println!("Circle address {:?}", usdc_address);
}
