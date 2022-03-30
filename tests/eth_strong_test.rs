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

use ethcontract::{
  errors::ExecutionError,
  transaction::gas_price::GasPrice,
  transport::DynTransport,
  web3::{
    types::{Address, Bytes, CallRequest, TransactionParameters, U256},
    Transport,
  },
  Account, Http, Web3, H160,
};
use iota_stronghold::{Location, ProcResult, Procedure, ResultMessage, Stronghold};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn main() {
  let chain_id = 1337;
  let web3 = Web3::new(DynTransport::new(
    Http::new("http://localhost:8545").expect("couldnt setup web3"),
  ));

  let (strong1, loc1, account1) = init_account(&web3, chain_id, 1).await;

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  // prefunded account
  let zero_account: Account<DynTransport> = Account::Local(accounts[0], None);
  let zero_address = zero_account.address();

  let n = 1000;
  let mut i = 0;
  while i < n {
    let strong1 = strong1.clone();
    let loc1 = loc1.clone();
    let account1 = account1.clone();
    let web3 = web3.clone();
    tokio::spawn(async move {
      // let gas_price = GasPrice::Standard.resolve_for_transaction_request(&web3).await.expect("cant get gas price");
      let options = TransactionOptions {
        to: Some(zero_address),
        gas: None,
        value: Some(U256::exp10(18)),
        data: Some(Bytes(Vec::new())),
        nonce: Some(U256::exp10(5)),
      };
      let tx = build_transaction_parameters(
        web3.clone(),
        account1.address(),
        Some(chain_id),
        GasPrice::Standard,
        options,
      )
      .await
      .expect("cant build tx opts");
      let accounts = web3.accounts();
      let res = strong1
        .web3_runtime_exec(Procedure::Web3SignTransaction {
          accounts,
          private_key: loc1,
          tx: tx,
        })
        .await;
      println!("SIGN NOW {}", i);
      match res {
        ProcResult::Web3SignTransaction(ResultMessage::Ok(_)) => {
          println!("=> {}", i);
        }
        _ => println!("wtf"),
      }
    });
    i = i + 1;
  }
  sleep(Duration::from_secs(10)).await;
}

/// Shared transaction options that are used when finalizing transactions into
/// either `TransactionRequest`s or raw signed transaction `Bytes`.
#[derive(Clone, Debug, Default)]
struct TransactionOptions {
  /// The receiver of the transaction.
  pub to: Option<Address>,
  /// The amount of gas to use for the transaction.
  pub gas: Option<U256>,
  /// The ETH value to send with the transaction.
  pub value: Option<U256>,
  /// The data for the transaction.
  pub data: Option<Bytes>,
  /// The transaction nonce.
  pub nonce: Option<U256>,
}

async fn build_transaction_parameters<T: Transport>(
  web3: Web3<T>,
  public_address: Address,
  chain_id: Option<u64>,
  gas_price: GasPrice,
  options: TransactionOptions,
) -> Result<TransactionParameters, ExecutionError> {
  let gas = resolve_gas_limit(&web3, public_address, gas_price, &options).await?;
  let gas_price = gas_price.resolve(&web3).await?;

  Ok(TransactionParameters {
    nonce: options.nonce,
    gas_price: Some(gas_price),
    gas,
    to: options.to,
    value: options.value.unwrap_or_default(),
    data: options.data.unwrap_or_default(),
    chain_id,
    transaction_type: None,
    access_list: None,
    max_fee_per_gas: None,
    max_priority_fee_per_gas: None,
  })
}

async fn resolve_gas_limit<T: Transport>(
  web3: &Web3<T>,
  from: Address,
  gas_price: GasPrice,
  options: &TransactionOptions,
) -> Result<U256, ExecutionError> {
  match options.gas {
    Some(value) => Ok(value),
    None => Ok(
      web3
        .eth()
        .estimate_gas(
          CallRequest {
            from: Some(from),
            to: options.to,
            gas: None,
            gas_price: gas_price.value(),
            value: options.value,
            data: options.data.clone(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
          },
          None,
        )
        .await?,
    ),
  }
}
pub async fn eth_balance(web3: &Web3<DynTransport>, addy: H160) -> u128 {
  web3
    .eth()
    .balance(addy, None)
    .await
    .expect("couldnt get safe ETH balance")
    .as_u128()
}

async fn init_account(
  web3: &Web3<DynTransport>,
  chain_id: u64,
  keynum: u8,
) -> (Stronghold, Location, Account<DynTransport>) {
  let (tx, rx) = std::sync::mpsc::channel();
  std::thread::spawn(move || {
    let system = actix::System::new();
    let stronghold = system
      .block_on(Stronghold::init_stronghold_system(b"path".to_vec(), vec![]))
      .unwrap();
    tx.send(stronghold).unwrap();
    system.run().expect("actix system run failed");
  });
  let stronghold = rx.recv().unwrap();

  let keypair_location = Location::generic("SECP256K1", "keypair");

  match stronghold
    .runtime_exec(Procedure::Secp256k1Store {
      key: [keynum; 32].to_vec(),
      output: keypair_location.clone(),
      hint: [0u8; 24].into(),
    })
    .await
  {
    ProcResult::Secp256k1Generate(ResultMessage::OK) => (),
    r => panic!("unexpected result: {:?}", r),
  }

  let accounts = web3.accounts();
  (
    stronghold.clone(),
    keypair_location.clone(),
    Account::Stronghold(stronghold, accounts, keypair_location, Some(chain_id)),
  )
}
