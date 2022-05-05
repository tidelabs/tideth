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
use ethcontract::{transport::DynTransport, Http, Web3};
use tideth::router::RouterClient;

ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");

const ROUTER_ADDRESS: &str = "0xe72b6a5f4cc34dfa68e08f82d6d16055a513842a";

#[tokio::main]
async fn main() {
  dotenv().ok();

  let eth_url: String = std::env::var("ETH_URL").unwrap_or("http://localhost:8545".to_string());
  let web3 = Web3::new(DynTransport::new(
    Http::new(eth_url.as_str()).expect("couldnt setup web3"),
  ));

  let router = RouterClient::new(&web3, Some(ROUTER_ADDRESS)).expect("derp");

  let pending_owner = router
    .pending_owner()
    .await
    .expect("couldnt call pending_owner");
  println!("PENDING OWNER {:?}", pending_owner);

  let owner = router.owner().await.expect("couldnt call owner");
  println!(" OWNER {:?}", owner);
}
