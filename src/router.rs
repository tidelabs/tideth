use crate::{error::Error, utils, Result};
use ethcontract::{
  prelude::*,
  tokens::Tokenize,
  transaction::TransactionResult,
  transport::DynTransport,
  web3::ethabi::{param_type::ParamType, Function, Param},
};
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};

ethcontract::contract!("./artifacts/contracts/Router.sol/Router.json");

#[derive(Debug, Clone)]
pub struct DepositEvent {
  pub amount: u128,
  pub asset: H160,
  pub account: [u8; 32],
  pub tx_hash: Vec<u8>,
  pub tx_index: usize, // log_index is the index of the log in the block
  pub blockheight: u64,
  pub confirmations: u64,
}
#[derive(Debug, Clone)]
pub struct WithdrawEvent {
  pub amount: u128,
  pub asset: H160,
  pub account: H160,
  pub tx_hash: Vec<u8>,
  pub blockheight: u64,
  pub confirmations: u64,
}

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Clone)]
pub struct RouterClient {
  web3: Web3<DynTransport>,
  address: Address,
}

impl RouterClient {
  pub fn new(web3: &Web3<DynTransport>, address: Option<&str>) -> Result<Self> {
    Ok(Self {
      address: utils::address_or_default(address)?,
      web3: web3.clone(),
    })
  }

  pub async fn block_number(&self) -> Result<u64> {
    let h = self.web3.eth().block_number().await?;
    Ok(h.as_u64())
  }

  pub fn at(&mut self, address: &str) -> Result<router::Contract> {
    self.address = utils::address_or_default(Some(address))?;
    let contract = Router::at(&self.web3, self.address);
    Ok(contract)
  }

  pub fn address(&self) -> H160 {
    self.address
  }

  pub fn set_address(&mut self, address: H160) {
    self.address = address;
  }

  pub async fn deploy(&mut self, from_account: Account<DynTransport>) -> Result<String> {
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    let router = Router::builder(&self.web3)
      //.gas(500_000_u64.into())
      .from(from_account)
      .nonce(nonce)
      .deploy()
      .await?;

    self.address = router.address();
    Ok(router.address().to_string())
  }

  pub async fn transfer_ownership(
    &mut self,
    from_account: Account<DynTransport>,
    new_owner: H160,
  ) -> Result<bool> {
    let router = Router::at(&self.web3, self.address);
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    router
      .transfer_ownership(new_owner)
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
    Ok(true)
  }

  pub async fn claim_ownership(&mut self, from_account: Account<DynTransport>) -> Result<bool> {
    let router = Router::at(&self.web3, self.address);
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    router
      .claim_ownership()
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
    Ok(true)
  }

  pub async fn owner(&self) -> Result<H160> {
    let router = Router::at(&self.web3, self.address);
    Ok(router.owner().call().await?)
  }

  pub async fn pending_owner(&self) -> Result<H160> {
    let router = Router::at(&self.web3, self.address);
    Ok(router.pending_owner().call().await?)
  }

  fn _make_data(&self, to: H160, asset: H160, amount: u128) -> Result<Vec<u8>> {
    #[allow(deprecated)]
    let withdrawal = Function {
      name: "withdraw".to_owned(),
      inputs: vec![
        Param {
          name: "".to_owned(),
          kind: ParamType::Address,
          internal_type: None,
        },
        Param {
          name: "".to_owned(),
          kind: ParamType::Address,
          internal_type: None,
        },
        Param {
          name: "".to_owned(),
          kind: ParamType::Uint(256),
          internal_type: None,
        },
      ],
      outputs: vec![Param {
        name: "".to_owned(),
        kind: ParamType::Bool,
        internal_type: None,
      }],
      constant: None,
      state_mutability: Default::default(),
    };

    let tx = withdrawal.encode_input(&vec![
      to.into_token(),
      asset.into_token(),
      U256::from(amount).into_token(),
    ]);

    if let Err(e) = tx {
      return Err(Error::Other(e.to_string()));
    }
    let data = tx.unwrap();
    Ok(data)
  }

  fn _make_claim_data(&self) -> Result<Vec<u8>> {
    #[allow(deprecated)]
    let claim = Function {
      name: "claimOwnership".to_owned(),
      inputs: vec![],
      outputs: vec![],
      constant: None,
      state_mutability: Default::default(),
    };
    let tx = claim.encode_input(&vec![]);
    if let Err(e) = tx {
      return Err(Error::Other(e.to_string()));
    }
    let data = tx.unwrap();
    Ok(data)
  }

  fn _make_accept_data(&self, token: Address) -> Result<Vec<u8>> {
    #[allow(deprecated)]
    let accept = Function {
      name: "acceptToken".to_owned(),
      inputs: vec![Param {
        name: "".to_owned(),
        kind: ParamType::Address,
        internal_type: None,
      }],
      outputs: vec![],
      constant: None,
      state_mutability: Default::default(),
    };
    let tx = accept.encode_input(&vec![token.into_token()]);
    if let Err(e) = tx {
      return Err(Error::Other(e.to_string()));
    }
    let data = tx.unwrap();
    Ok(data)
  }

  pub fn eth_withdrawal_data(&self, to: H160, amount: u128) -> Result<Vec<u8>> {
    let eth: Address = utils::zero_address();
    self._make_data(to, eth, amount)
  }

  pub fn erc20_withdrawal_data(&self, to: H160, asset: H160, amount: u128) -> Result<Vec<u8>> {
    self._make_data(to, asset, amount)
  }

  pub fn claim_ownership_data(&self) -> Result<Vec<u8>> {
    self._make_claim_data()
  }

  pub fn accept_token_data(&self, asset: Address) -> Result<Vec<u8>> {
    self._make_accept_data(asset)
  }

  pub async fn deposit(
    &self,
    from_account: Account<DynTransport>,
    account: Vec<u8>, // tidechain account
    asset: H160,
    amount: U256,
    value: Option<U256>,
  ) -> Result<(Vec<u8>, u128)> {
    if account.len() != 32 {
      return Err(Error::Other("wrong account length".to_string()));
    }

    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;

    let mut arr = [0u8; 32];
    for i in 0..account.len() {
      arr[i] = account[i];
    }

    let router = Router::at(&self.web3, self.address);
    let call = if let Some(ea) = value {
      if ea != amount {
        return Err(Error::Other("invalid value".to_string()));
      }
      router
        .deposit(Bytes(arr), asset, amount)
        .from(from_account)
        .nonce(nonce)
        .value(ea)
    } else {
      router
        .deposit(Bytes(arr), asset, amount)
        .from(from_account)
        .nonce(nonce)
    };
    let tx_result = call.send().await?;
    Ok(match tx_result {
      TransactionResult::Hash(h) => (h.0.to_vec(), 0), // should not ever happen
      TransactionResult::Receipt(r) => {
        let gas = if let Some(g) = r.gas_used {
          g.as_u128()
        } else {
          0
        };
        (r.transaction_hash.0.to_vec(), gas)
      }
    })
  }

  pub async fn add_token(&self, from_account: Account<DynTransport>, asset: H160) -> Result<()> {
    let router = Router::at(&self.web3, self.address);
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    router
      .accept_token(asset)
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
    Ok(())
  }

  pub async fn remove_token(&self, from_account: Account<DynTransport>, asset: H160) -> Result<()> {
    let router = Router::at(&self.web3, self.address);
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    router
      .remove_token(asset)
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
    Ok(())
  }

  pub async fn is_accepted(&self, asset: H160) -> Result<bool> {
    let router = Router::at(&self.web3, self.address);
    let is = router.is_accepted(asset).call().await?;
    Ok(is)
  }

  pub async fn get_deposits_by_account(
    &self,
    account: [u8; 32],
    since: Option<u64>,
  ) -> Result<Vec<DepositEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router
      .all_events()
      .from_block(bn)
      .topic1(Topic::This(account.into()))
      .query()
      .await?;
    Ok(self.events_to_deposits(events).await)
  }

  pub async fn print_all_logs(&self, since: Option<u64>) -> Result<()> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router.all_events().from_block(bn).query().await?;
    println!("ALL EVENTS: {:?}", events);
    Ok(())
  }

  pub async fn get_all_deposits(&self, since: Option<u64>) -> Result<Vec<DepositEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router.all_events().from_block(bn).query().await?;
    Ok(self.events_to_deposits(events).await)
  }

  pub async fn get_deposits_by_asset(
    &self,
    asset: Address,
    since: Option<u64>,
  ) -> Result<Vec<DepositEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router
      .all_events()
      .from_block(bn)
      .topic2(Topic::This(asset.into()))
      .query()
      .await?;
    Ok(self.events_to_deposits(events).await)
  }

  pub async fn get_withdraws_by_account(
    &self,
    account: Address,
    since: Option<u64>,
  ) -> Result<Vec<WithdrawEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router
      .all_events()
      .from_block(bn)
      .topic1(Topic::This(account.into()))
      .query()
      .await?;
    Ok(self.events_to_withdrawals(events).await)
  }

  pub async fn get_all_withdraws(&self, since: Option<u64>) -> Result<Vec<WithdrawEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router.all_events().from_block(bn).query().await?;
    Ok(self.events_to_withdrawals(events).await)
  }

  pub async fn get_withdraws_by_asset(
    &self,
    asset: Address,
    since: Option<u64>,
  ) -> Result<Vec<WithdrawEvent>> {
    let router = Router::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = router
      .all_events()
      .from_block(bn)
      .topic2(Topic::This(asset.into()))
      .query()
      .await?;
    Ok(self.events_to_withdrawals(events).await)
  }

  pub async fn subscribe_events(&self, mut tx: Sender<DepositEvent>) {
    let router = Router::at(&self.web3, self.address);
    let mut deposits = router
      .events()
      .deposit()
      // .from(Topic::This(accounts[0]))
      .stream()
      .boxed();
    loop {
      let d_opt = deposits.next().await;
      let bn = match self.block_number().await {
        Ok(n) => Some(n),
        Err(_) => None,
      };
      if let Some(d_res) = d_opt {
        if let Ok(d) = d_res {
          let dep_opt = d.clone().added();
          // let dep_meta = d.meta;
          if let Some(dep) = dep_opt {
            if let Some(meta) = d.meta {
              let confs = if let Some(n) = bn {
                n - meta.block_number
              } else {
                0
              };
              let de: DepositEvent = DepositEvent {
                amount: dep.amount.as_u128(),
                asset: dep.asset,
                account: dep.account.0,
                tx_hash: meta.transaction_hash.as_bytes().to_vec(),
                blockheight: meta.block_number,
                tx_index: meta.log_index,
                confirmations: confs,
              };
              tx.send(de).await.expect("COULDNT SEND DEPOSIT EVENT");
            }
            format!("DEPOSIT {:?} {:?} {:?}", dep.amount, dep.asset, dep.account);
          }
        }
      }
    }
  }

  async fn events_to_deposits(&self, events: Vec<Event<router::Event>>) -> Vec<DepositEvent> {
    let bn = match self.block_number().await {
      Ok(n) => Some(n),
      Err(_) => None,
    };
    let mut deps: Vec<DepositEvent> = vec![];
    events.iter().for_each(|e| {
      if let router::Event::Deposit(dep) = &e.data {
        if let Some(meta) = &e.meta {
          let confs = if let Some(n) = bn {
            n - meta.block_number
          } else {
            0
          };
          deps.push(DepositEvent {
            amount: dep.amount.as_u128(),
            account: dep.account.0,
            asset: dep.asset,
            tx_hash: meta.transaction_hash.as_bytes().to_vec(),
            blockheight: meta.block_number,
            tx_index: meta.log_index,
            confirmations: confs,
          });
        }
      }
    });
    deps
  }

  async fn events_to_withdrawals(&self, events: Vec<Event<router::Event>>) -> Vec<WithdrawEvent> {
    let bn = match self.block_number().await {
      Ok(n) => Some(n),
      Err(_) => None,
    };
    let mut deps: Vec<WithdrawEvent> = vec![];
    events.iter().for_each(|e| {
      if let router::Event::Withdraw(dep) = &e.data {
        if let Some(meta) = &e.meta {
          let confs = if let Some(n) = bn {
            n - meta.block_number
          } else {
            0
          };
          deps.push(WithdrawEvent {
            amount: dep.amount.as_u128(),
            account: dep.account,
            asset: dep.asset,
            tx_hash: meta.transaction_hash.as_bytes().to_vec(),
            blockheight: meta.block_number,
            confirmations: confs,
          });
        }
      }
    });
    deps
  }
}
