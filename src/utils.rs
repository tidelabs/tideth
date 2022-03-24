use crate::Result;
use ethcontract::prelude::{Address, H160};
use std::str::FromStr;

pub fn zero_address() -> Address {
  "0x0000000000000000000000000000000000000000"
    .parse()
    .unwrap()
}

pub fn address_or_default(address: Option<&str>) -> Result<H160> {
  if let Some(a) = address {
    Ok(H160::from_str(a)?)
  } else {
    Ok(zero_address())
  }
}
