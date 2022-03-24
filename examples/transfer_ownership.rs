use dotenv::dotenv;
use ethcontract::{transport::DynTransport, Account, Http, PrivateKey, Web3};
use std::str::FromStr;
use tideth::{router::RouterClient, safe::SafeClient};

ethcontract::contract!("./artifacts/contracts/RustCoin.sol/RustCoin.json");

const ROUTER_ADDRESS: &str = "0xae8a6463bf8449e6b5ee8277924cd6132b809be4";
const SAFE_ADDRESS: &str = "0x971c11eb24778bf6824c82f0e82d6530bdeff7a2";

#[tokio::main]
async fn main() {
  dotenv().ok();

  // chain id
  let chain_id: u64 = std::env::var("CHAIN_ID")
    .unwrap_or("1337".to_string())
    .parse::<u64>()
    .unwrap_or(1337);

  // private key from metamask
  let priv_key_string = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
  let priv_key = PrivateKey::from_str(priv_key_string.as_str()).expect("couldnt parse private key");

  let eth_url: String = std::env::var("ETH_URL").unwrap_or("http://localhost:8545".to_string());
  let web3 = Web3::new(DynTransport::new(
    Http::new(eth_url.as_str()).expect("couldnt setup web3"),
  ));

  // either imported account from metamask
  let my_account = Account::Offline(priv_key, Some(chain_id));
  let my_address = my_account.address();
  println!("my address {:?}", my_address);

  // check my balance
  let my_eth_balance = web3
    .eth()
    .balance(my_address, None)
    .await
    .expect("couldnt get ETH balance");

  if my_eth_balance.as_u128() == 0 {
    panic!("no eth for fees");
  }

  //
  let mut router = RouterClient::new(&web3, Some(ROUTER_ADDRESS)).expect("derp");

  let pending_owner = router
    .pending_owner()
    .await
    .expect("couldnt call pending_owner");
  println!("PENDING OWNER {:?}", pending_owner);

  let current_owner = router.owner().await.expect("couldnt get currenct owner");

  let safe = SafeClient::new(&web3, Some(SAFE_ADDRESS)).expect("derp");
  // transfer router to safe
  if current_owner != safe.address() {
    router
      .transfer_ownership(my_account.clone(), safe.address())
      .await
      .expect("Couldnt transfer router ownership to safe");

    println!("safe is pending_owner of router");
  } else {
    println!("safe already owns router");
  }

  let pending_owner = router
    .pending_owner()
    .await
    .expect("couldnt call pending_owner");
  assert_eq!(
    pending_owner,
    safe.address(),
    "safe should be able to claim router"
  );
}
