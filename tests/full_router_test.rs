use ethcontract::{
  transport::DynTransport,
  web3::types::{TransactionRequest, U256},
  Account, Bytes, Http, Web3, H160,
};
use futures::{join, stream::StreamExt};
use std::str::FromStr;

ethcontract::contract!("artifacts/contracts/FeeCoin.sol/ERC20.json");
ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");
ethcontract::contract!("artifacts/contracts/Router.sol/Router.json");

const ETH_URL: &str = "http://localhost:8545";

#[tokio::test]
async fn main() {
  let web3 = Web3::new(DynTransport::new(
    Http::new(ETH_URL).expect("couldnt setup web3"),
  ));

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  // prefunded test account
  let zero_account = Account::Local(accounts[0], None);
  let one_account = Account::Local(accounts[1], None);
  let two_account = Account::Local(accounts[2], None);

  // deploy the router contract
  let router = Router::builder(&web3)
    //.gas(500_000_u64.into())
    .from(one_account.clone())
    .deploy()
    .await
    .expect("couldnt deploy router");
  println!("Router address {:?}", router.address());

  test_direct_tx(&router, &web3).await;

  test_safe_ownable(&router, zero_account.clone(), one_account.clone()).await;

  // Deploy an ERC20
  let erc20 = RustCoin::builder(&web3)
    //.gas(1_000_000_u64.into())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  test_acceptlist(&router, &erc20, zero_account.clone()).await;

  test_deposits(
    &router,
    &erc20,
    zero_account.clone(),
    one_account.clone(),
    &web3,
  )
  .await;

  test_withdrawals(
    &router,
    &erc20,
    zero_account.clone(),
    one_account,
    two_account,
    &web3,
  )
  .await;

  test_fee_on_transfer(&router, zero_account, &web3).await;
}

async fn test_direct_tx(router: &router::Contract, web3: &Web3<DynTransport>) {
  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  // try to send ETH directly to the router (should fail)
  let tx_object = TransactionRequest {
    from: accounts[0],
    to: Some(router.address()),
    value: Some(U256::exp10(17)), //0.1 eth
    ..Default::default()
  };
  let raw_tx_res = web3.eth().send_transaction(tx_object).await;
  if let Ok(_) = raw_tx_res {
    panic!("ETH tx to router should fail");
  }
}

async fn test_safe_ownable(
  router: &router::Contract,
  zero_account: Account<DynTransport>,
  one_account: Account<DynTransport>,
) {
  // try to claim ownership by accounts[0] (should fail)
  let ownership_claimed = router
    .claim_ownership()
    .from(zero_account.clone())
    .send()
    .await;
  if let Ok(_) = ownership_claimed {
    panic!("claim ownership should fail");
  }

  // transfer to accounts[0]
  router
    .transfer_ownership(zero_account.address())
    .from(one_account.clone())
    .send()
    .await
    .expect("should transfer ownership");

  // check the owner
  let pending = router
    .pending_owner()
    .call()
    .await
    .expect("should got owner");
  assert!(
    pending == zero_account.address(),
    "accounts[0] should be pending"
  );

  // accounts[0] claims
  router
    .claim_ownership()
    .from(zero_account.clone())
    .send()
    .await
    .expect("ownership should be claimable");

  // check the owner
  let own = router.owner().call().await.expect("should got owner");
  assert!(own == zero_account.address(), "accounts[0] should own");
}

async fn test_acceptlist(
  router: &router::Contract,
  erc20: &rust_coin::Contract,
  zero_account: Account<DynTransport>,
) {
  // deposit 100 tokens to the router (without acceptlisting, should fail)
  let deposit_res = router
    .deposit(Bytes(empty_account()), router.address(), 100.into())
    .send()
    .await;
  if let Ok(_) = deposit_res {
    panic!("deposit should fail");
  }

  let mut accepted_events = router.events().accepted().stream().boxed();
  join! {
    async {
    // add to the acceptlist
    router
      .accept_token(erc20.address())
      .from(zero_account.clone())
      .send()
      .await
      .expect("couldnt add to accept list");
    },
    async {
      let accepted = accepted_events.next()
        .await
        .expect("no more events")
        .expect("error querying event")
        .added()
        .expect("expected added event");
        assert_eq!(accepted.asset, erc20.address(), "wrong asset");
    },
  };

  // check it was added
  let is_accepted = router
    .is_accepted(erc20.address())
    .call()
    .await
    .expect("couldnt call is_accepted");
  assert_eq!(is_accepted, true);

  let mut removed_events = router.events().removed().stream().boxed();
  join! {
    async {
    // add to the acceptlist
    router
      .remove_token(erc20.address())
      .from(zero_account.clone())
      .send()
      .await
      .expect("couldnt add to accept list");
    },
    async {
      let removed = removed_events.next()
        .await
        .expect("no more events")
        .expect("error querying event")
        .added()
        .expect("expected added event");
        assert_eq!(removed.asset, erc20.address(), "wrong asset");
    },
  };

  // check it was removed
  let is_accepted = router
    .is_accepted(erc20.address())
    .call()
    .await
    .expect("couldnt call is_accepted");
  assert_eq!(is_accepted, false);

  // deposit 100 tokens to the router (after removing, should fail)
  let deposit_res = router
    .deposit(Bytes(empty_account()), router.address(), 100.into())
    .send()
    .await;
  if let Ok(_) = deposit_res {
    panic!("deposit should fail");
  }

  // add it back in to continue the test
  router
    .accept_token(erc20.address())
    .from(zero_account.clone())
    .send()
    .await
    .expect("couldnt add to accept list");
}

fn empty_account() -> [u8; 32] {
  let empty_substrate_addy =
    hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
      .expect("couldnt decode empty substrate addy");
  let mut empty_addy: [u8; 32] = Default::default();
  empty_addy[..].copy_from_slice(&empty_substrate_addy[0..32]);
  empty_addy
}

async fn test_deposits(
  router: &router::Contract,
  erc20: &rust_coin::Contract,
  zero_account: Account<DynTransport>,
  one_account: Account<DynTransport>,
  web3: &Web3<DynTransport>,
) {
  let coin_amt = 100;

  erc20
    .approve(router.address(), coin_amt.into())
    .send()
    .await
    .expect("couldnt approve");

  // try the deposit again (and check emitted event)
  let mut deposits = router.events().deposit().stream().boxed();
  join! {
    async {
      router
      .deposit(Bytes(empty_account()), erc20.address(), coin_amt.into())
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
      assert_eq!(deposit.amount.as_u128(), coin_amt, "wrong amount");
      assert_eq!(deposit.asset, erc20.address(), "wrong asset");
      assert_eq!(deposit.account.0, empty_account(), "wrong account");
    },
  };

  assert_eq!(
    erc20
      .balance_of(router.address())
      .call()
      .await
      .expect("Couldn't get router's balance"),
    coin_amt.into()
  );

  println!(
    "Router RustCoin balance = {:?}! {:?}. ERC20 deposit successful!",
    coin_amt,
    router.address()
  );

  let before_balance = web3
    .eth()
    .balance(zero_account.address(), None)
    .await
    .expect("couldnt get ETH balance");

  // Send ETH
  let eth_amt: u128 = 150000000000;
  let eth_addy_str = "0x0000000000000000000000000000000000000000";
  let eth_addy = H160::from_str(eth_addy_str).expect("couldnt parse zero address");

  // deposits event stream
  join! {
    async {
     // send with proper amount
      router
      .deposit(Bytes(empty_account()), eth_addy.into(), eth_amt.into())
      .value(eth_amt.into())
      .from(one_account.clone())
      .send()
      .await
      .expect("couldnt deposit ETH");
    },
    async {
      let deposit = deposits.next()
          .await
          .expect("no more events")
          .expect("error querying event")
          .added()
          .expect("expected added event");
      assert_eq!(deposit.amount.as_u128(), eth_amt, "wrong amount");
      assert_eq!(deposit.asset, eth_addy, "wrong asset");
      assert_eq!(deposit.account.0, empty_account(), "wrong account");
    },
  };

  let after_balance = web3
    .eth()
    .balance(zero_account.address(), None)
    .await
    .expect("couldnt get ETH balance");

  assert_eq!(
    before_balance.as_u128(),
    after_balance.as_u128() - eth_amt,
    "depositted ETH should go to contract owner"
  );

  println!("ETH depositted!");

  // send with no amount (should fail)
  let eth_deposit_res = router
    .deposit(Bytes(empty_account()), eth_addy.into(), eth_amt.into())
    .from(one_account.clone())
    .send()
    .await;
  if let Ok(_) = eth_deposit_res {
    panic!("should fail because no amount sent");
  }
}

async fn test_withdrawals(
  router: &router::Contract,
  erc20: &rust_coin::Contract,
  zero_account: Account<DynTransport>,
  one_account: Account<DynTransport>,
  two_account: Account<DynTransport>,
  web3: &Web3<DynTransport>,
) {
  let withdraw_amt: u128 = 100;
  let eth_addy_str = "0x0000000000000000000000000000000000000000";
  let eth_addy = H160::from_str(eth_addy_str).expect("couldnt parse zero address");

  // test withdrawal by non-owner (should fail)
  let eth_withdrawal_res = router
    .withdraw(two_account.address(), erc20.address(), withdraw_amt.into())
    .from(one_account.clone())
    .send()
    .await;
  if let Ok(_) = eth_withdrawal_res {
    panic!("should fail because not owner");
  }

  // test ERC20 withdrawal
  let mut withdrawals = router.events().withdraw().stream().boxed();
  join! {
    async {
      router
      .withdraw(two_account.address(), erc20.address(), withdraw_amt.into())
      .from(zero_account.clone())
      .send()
      .await
      .expect("couldnt withdraw again");
    },
    async {
      let w = withdrawals.next()
        .await
        .expect("no more events")
        .expect("error querying event")
        .added()
        .expect("expected added event");
      assert_eq!(w.amount.as_u128(), withdraw_amt, "wrong amount");
      assert_eq!(w.asset, erc20.address(), "wrong asset");
      assert_eq!(w.account, two_account.address(), "wrong account");
    },
  };

  // withdrawal recipient got the ERC20
  assert_eq!(
    erc20
      .balance_of(two_account.address())
      .call()
      .await
      .expect("Couldn't get router's balance"),
    withdraw_amt.into()
  );

  println!("ERC20 has been withdrawn!");

  let before_balance2 = web3
    .eth()
    .balance(two_account.address(), None)
    .await
    .expect("couldnt get ETH balance");

  // test ETH withdrawal
  join! {
    async {
      router
      .withdraw(two_account.address(), eth_addy.into(), withdraw_amt.into())
      .value(withdraw_amt.into())
      .from(zero_account.clone())
      .send()
      .await
      .expect("couldnt withdraw again");
    },
    async {
      let w = withdrawals.next()
        .await
        .expect("no more events")
        .expect("error querying event")
        .added()
        .expect("expected added event");
      assert_eq!(w.amount.as_u128(), withdraw_amt, "wrong amount");
      assert_eq!(w.asset, eth_addy.into(), "not ETH");
      assert_eq!(w.account, two_account.address(), "wrong account");
    },
  };

  let after_balance2 = web3
    .eth()
    .balance(two_account.address(), None)
    .await
    .expect("couldnt get ETH balance");

  assert_eq!(
    before_balance2.as_u128(),
    after_balance2.as_u128() - withdraw_amt,
    "ETH should go to accounts[2]"
  );

  println!("ETH has been withdrawn!");
}

async fn test_fee_on_transfer(
  router: &router::Contract,
  zero_account: Account<DynTransport>,
  web3: &Web3<DynTransport>,
) {
  // Deploy an ERC20 with fee-on-transfer
  let feecoin = ERC20::builder(&web3, "FeeCoin".to_string(), "FEE".to_string())
    //.gas(1_000_000_u64.into())
    .deploy()
    .await
    .expect("Couldn't deploy the FeeCoin");

  let coin_amt = 100;
  let fee = 42;

  feecoin
    .approve(router.address(), coin_amt.into())
    .send()
    .await
    .expect("couldnt approve");

  router
    .accept_token(feecoin.address())
    .from(zero_account.clone())
    .send()
    .await
    .expect("couldnt add to accept list");

  let final_amount = coin_amt - fee;

  // try the deposit again (and check emitted event)
  let mut deposits = router.events().deposit().stream().boxed();
  join! {
    async {
      router
      .deposit(Bytes(empty_account()), feecoin.address(), coin_amt.into())
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
      assert_eq!(deposit.amount.as_u128(), final_amount, "wrong amount");
      assert_eq!(deposit.asset, feecoin.address(), "wrong asset");
      assert_eq!(deposit.account.0, empty_account(), "wrong account");
    },
  };

  assert_eq!(
    feecoin
      .balance_of(router.address())
      .call()
      .await
      .expect("Couldn't get router's balance"),
    final_amount.into()
  );

  println!(
    "Router FeeCoin balance = {:?}! {:?}. FeeCoin deposit successful!",
    final_amount,
    router.address()
  );
}
