use crate::{error::Error, utils, Result};
use ethcontract::{
  prelude::*,
  tokens::Tokenize,
  transaction::TransactionResult,
  transport::DynTransport,
  web3::ethabi::{param_type::ParamType, Function, Param},
  Bytes,
};

ethcontract::contract!("artifacts/contracts/GnosisSafe.sol/GnosisSafe.json");
ethcontract::contract!("artifacts/contracts/GnosisSafeProxy.sol/GnosisSafeProxy.json");

#[derive(Clone)]
pub struct SafeClient {
  web3: Web3<DynTransport>,
  address: Address,
}

#[derive(Clone)]
pub struct ExecutionSuccess {
  pub block_number: u64,
  pub log_index: usize,
  pub tx_hash: Vec<u8>,
  pub amount: U256,
  pub inner_tx_hash: Vec<u8>, // the tx_hash of the executed SAFE tx (of the router withdrawal for example)
}

impl SafeClient {
  pub fn new(web3: &Web3<DynTransport>, address: Option<&str>) -> Result<Self> {
    Ok(Self {
      address: utils::address_or_default(address)?,
      web3: web3.clone(),
    })
  }

  pub fn address(&self) -> H160 {
    self.address
  }

  pub fn set_address(&mut self, address: H160) {
    self.address = address;
  }

  pub async fn deploy(&mut self, account: Account<DynTransport>) -> Result<String> {
    // Deploy the Gnosis Safe contract
    let singleton = GnosisSafe::builder(&self.web3)
      // .gas(6_000_000u64.into())
      .from(account.clone())
      .deploy()
      .await?;
    // Any real deployment would use the proxy factory to create a new proxy
    // This is solely simpler to test with
    let proxy = GnosisSafeProxy::builder(&self.web3, singleton.address())
      //.gas(500_000_u64.into())
      .from(account)
      .deploy()
      .await?;

    self.address = proxy.address();
    Ok(proxy.address().to_string())
  }

  pub async fn nonce(&self) -> Result<u64> {
    let safe = GnosisSafe::at(&self.web3, self.address);
    let n = safe.nonce().call().await?;
    Ok(n.as_u64())
  }

  pub async fn setup(
    &mut self,
    from_account: Account<DynTransport>,
    owners: Vec<H160>,
    threshold: u64,
  ) -> Result<bool> {
    let address_0: Address = utils::zero_address();
    let safe = GnosisSafe::at(&self.web3, self.address);
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    safe
      .setup(
        // Owners and threshold
        owners,
        threshold.into(),
        // Callback/expanded functionality not used
        address_0,
        Bytes::<Vec<u8>>(vec![]),
        address_0,
        address_0,
        0_u64.into(),
        address_0,
      )
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
    Ok(true)
  }

  pub async fn encode_data(
    &self,
    address: Address,
    amount: u128,
    data: Vec<u8>,
    nonce: u64,
  ) -> Result<Bytes<Vec<u8>>> {
    let safe = GnosisSafe::at(&self.web3, self.address);
    let address_0: Address = utils::zero_address();
    // Get the hash of the transaction according to how Gnosis will handle it
    let tx_data = safe
      .encode_transaction_data(
        // TX
        address,
        amount.into(),
        Bytes(data),
        0,
        // Refund data
        0_u64.into(),
        0_u64.into(),
        0_u64.into(),
        address_0,
        address_0,
        // Nonce
        nonce.into(),
      )
      .call()
      .await?;
    Ok(tx_data)
  }

  fn _make_erc20_data(&self, to: H160, amount: u128) -> Result<Vec<u8>> {
    #[allow(deprecated)]
    let erc20_transfer = Function {
      name: "transfer".to_owned(),
      inputs: vec![
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

    let tx = erc20_transfer.encode_input(&vec![to.into_token(), U256::from(amount).into_token()]);

    if let Err(e) = tx {
      return Err(Error::Other(e.to_string()));
    }
    let data = tx.unwrap();
    Ok(data)
  }

  pub async fn encode_erc20_tx(
    &self,
    erc20_address: H160,
    to: H160,
    amount: u128,
    nonce: u64,
  ) -> Result<Bytes<Vec<u8>>> {
    let data = self._make_erc20_data(to, amount)?;
    Ok(self.encode_data(erc20_address, 0, data, nonce).await?)
  }

  pub async fn encode_eth_tx(&self, to: H160, amount: u128, nonce: u64) -> Result<Bytes<Vec<u8>>> {
    Ok(self.encode_data(to, amount, Vec::new(), nonce).await?)
  }

  pub async fn exec_erc20_tx(
    &self,
    from_account: Account<DynTransport>,
    erc20_address: H160,
    to: Address,
    amount: u128,
    signatures: Vec<u8>,
  ) -> Result<(Vec<u8>, u128)> {
    let data = self._make_erc20_data(to, amount)?;
    let safe = GnosisSafe::at(&self.web3, self.address);
    Ok(
      self
        ._exec(&safe, from_account, erc20_address, 0, data, signatures)
        .await?,
    )
  }

  pub async fn exec_eth_tx(
    &self,
    from_account: Account<DynTransport>,
    to: Address,
    amount: u128,
    signatures: Vec<u8>,
  ) -> Result<(Vec<u8>, u128)> {
    let safe = GnosisSafe::at(&self.web3, self.address);
    Ok(
      self
        ._exec(&safe, from_account, to, amount, Vec::new(), signatures)
        .await?,
    )
  }

  pub async fn exec_with_data(
    &self,
    from_account: Account<DynTransport>,
    address: H160,
    data: Vec<u8>,
    signatures: Vec<u8>,
    value: u128,
  ) -> Result<(Vec<u8>, u128)> {
    let safe = GnosisSafe::at(&self.web3, self.address);
    Ok(
      self
        ._exec(&safe, from_account, address, value, data, signatures)
        .await?,
    )
  }

  async fn _exec(
    &self,
    safe: &gnosis_safe::Contract,
    from_account: Account<DynTransport>,
    to: Address,
    amount: u128,
    data: Vec<u8>,
    signatures: Vec<u8>,
  ) -> Result<(Vec<u8>, u128)> {
    let nonce = self
      .web3
      .eth()
      .transaction_count(from_account.address(), None)
      .await?;
    // let gas_price = self.web3.eth().gas_price().await?;
    let address_0: Address = utils::zero_address();
    let tx_result = safe
      .exec_transaction(
        to,
        amount.into(),
        Bytes(data),
        0,
        0_u64.into(),
        0_u64.into(),
        0_u64.into(),
        address_0,
        address_0,
        // Nonce isn't included as it's a SC global
        // Signatures
        Bytes(signatures),
      )
      .from(from_account)
      .nonce(nonce)
      .send()
      .await?;
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

  pub async fn get_execution_logs(&self, since: Option<u64>) -> Result<Vec<ExecutionSuccess>> {
    let safe = GnosisSafe::at(&self.web3, self.address);
    let bn: BlockNumber = match since {
      Some(s) => s.into(),
      None => BlockNumber::Earliest,
    };
    let events = safe.all_events().from_block(bn).query().await?;
    Ok(self.events_to_execution_successes(events).await)
  }

  async fn events_to_execution_successes(
    &self,
    events: Vec<Event<gnosis_safe::Event>>,
  ) -> Vec<ExecutionSuccess> {
    let mut ret: Vec<ExecutionSuccess> = vec![];
    events.iter().for_each(|e| {
      if let gnosis_safe::Event::ExecutionSuccess(exe) = &e.data {
        if let Some(meta) = &e.meta {
          ret.push(ExecutionSuccess {
            tx_hash: meta.transaction_hash.0.to_vec(),
            block_number: meta.block_number,
            log_index: meta.log_index,
            amount: exe.payment,
            inner_tx_hash: exe.tx_hash.0.to_vec(),
          });
        }
      }
    });
    ret
  }
}
