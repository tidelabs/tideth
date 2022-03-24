use ethcontract::{transport::DynTransport, Account, BlockNumber, Bytes, Http, Topic, Web3, H160};
use futures::{join, stream::StreamExt};

#[derive(Debug)]
pub struct DepositEvent {
  pub amount: u128,
  pub asset: H160,
  pub account: [u8; 32],
  pub tx_hash: Vec<u8>,
  pub blockheight: u64,
}

ethcontract::contract!("./artifacts/contracts/RustCoin.sol/RustCoin.json");
ethcontract::contract!("./artifacts/contracts/Router.sol/Router.json");

#[tokio::test]
async fn main() {
  let web3 = Web3::new(DynTransport::new(
    Http::new("http://localhost:8545").expect("couldnt setup web3"),
  ));

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  let zero_account = Account::Local(accounts[0], None);

  // Deploy an ERC20 and send 100 tokens to the safe
  let erc20 = RustCoin::builder(&web3)
    //.gas(1_000_000_u64.into())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  // deploy the router contract
  let router = Router::builder(&web3)
    //.gas(500_000_u64.into())
    .deploy()
    .await
    .expect("couldnt deploy router");
  println!("Router address {:?}", router.address());

  let coin_amt = 100;
  erc20
    .approve(router.address(), coin_amt.into())
    .send()
    .await
    .expect("couldnt approve");

  let empty_substrate_addy =
    hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
      .expect("couldnt decode empty substrate addy");
  let mut empty_addy: [u8; 32] = Default::default();
  empty_addy[..].copy_from_slice(&empty_substrate_addy[0..32]);

  // add to the acceptlist
  router
    .accept_token(erc20.address())
    .from(zero_account.clone())
    .send()
    .await
    .expect("couldnt add to accept list");

  // try the deposit again (and check emitted event)
  let mut deposits = router
    .events()
    .deposit()
    // .account(Topic::This(Bytes(empty_addy)))
    .stream()
    .boxed();
  join! {
    async {
      router
      .deposit(Bytes(empty_addy), erc20.address(), coin_amt.into())
      .from(zero_account.clone())
      .send()
      .await
      .expect("couldnt deposit again");
    },
    async {
      let deposit = deposits.next()
        .await
        .expect("no more events")
        .expect("error querying event")
        .added()
        .expect("expected added event");
      println!("got deposit! {:?}", deposit.amount.as_u128());
      assert_eq!(deposit.amount.as_u128(), coin_amt, "wrong amount");
      assert_eq!(deposit.asset, erc20.address(), "wrong asset");
      assert_eq!(deposit.account.0, empty_addy, "wrong account");
    },
  };

  println!("try query events");

  let event_history_vec = router
    .all_events()
    .from_block(BlockNumber::Earliest)
    .topic1(Topic::This(empty_addy.into()))
    .query()
    .await
    .expect("Couldn't retrieve event history");
  println!("Events 2: {:}", event_history_vec.len());

  println!("EVENTS {:?}", event_history_vec);

  let mut deps: Vec<DepositEvent> = vec![];
  // let deps: Vec<&Event<router::Event>> = event_history_vec
  event_history_vec.iter().for_each(|e| {
    if let router::Event::Deposit(dep) = &e.data {
      if let Some(meta) = &e.meta {
        // let meta = e.meta.clone();
        deps.push(DepositEvent {
          amount: dep.amount.as_u128(),
          account: dep.account.0,
          asset: dep.asset,
          tx_hash: meta.transaction_hash.as_bytes().to_vec(),
          blockheight: meta.block_number,
        });
      }
    }
  });
  println!("DEPS {:?}", deps);
}
