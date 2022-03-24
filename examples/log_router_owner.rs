use dotenv::dotenv;
use ethcontract::{transport::DynTransport, Http, Web3};
use tideth::router::RouterClient;

ethcontract::contract!("./artifacts/contracts/RustCoin.sol/RustCoin.json");

const ROUTER_ADDRESS: &str = "0xaa57cd19ae5ed73ea4be754051eb5933d1efd7e0";

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
