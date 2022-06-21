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
use ethcontract::{prelude::*, transport::DynTransport, Address};

ethcontract::contract!("https://tidefi-contracts.s3.eu-west-1.amazonaws.com/Tether.json");

pub async fn balance_of(
  web3: &Web3<DynTransport>,
  asset_address: Address,
  address: Address,
) -> Result<u128> {
  let tether = Tether::at(&web3, asset_address);
  let bal = tether.balance_of(address).call().await?;
  Ok(bal.as_u128())
}
