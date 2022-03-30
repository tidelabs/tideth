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
