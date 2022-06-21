use crate::Result;
use ethcontract::{prelude::*, transport::DynTransport};
use std::str::FromStr;

ethcontract::contract!("https://tidefi-contracts.s3.eu-west-1.amazonaws.com/Tether.json");

pub async fn balance_of(
  web3: &Web3<DynTransport>,
  asset_address: &str,
  address: &str,
) -> Result<u128> {
  let asset_addy = H160::from_str(asset_address)?;
  let tether = Tether::at(&web3, asset_addy);
  let addy = H160::from_str(address)?;
  let bal = tether.balance_of(addy).call().await?;
  Ok(bal.as_u128())
}
