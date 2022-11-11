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

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("web3 error: {0}")]
  Web3Error(#[from] ethcontract::web3::Error),
  #[error("contract error: {0}")]
  MethodError(#[from] ethcontract::errors::MethodError),
  #[error("deployment error: {0}")]
  DeployError(#[from] ethcontract::errors::DeployError),
  #[error("execution error: {0}")]
  ExecutionError(#[from] ethcontract::errors::ExecutionError),
  #[error("hex error: {0}")]
  HexError(#[from] hex::FromHexError),
  #[error("hex error: {0}")]
  CHexError(#[from] rustc_hex::FromHexError),
  #[error("ConfirmationTimeout: {0}")]
  ConfirmationTimeout(String),
  #[error("error: {0}")]
  Other(String),
}

impl From<&str> for Error {
  fn from(err: &str) -> Self {
    Error::Other(err.to_string())
  }
}
impl From<String> for Error {
  fn from(err: String) -> Self {
    Error::Other(err)
  }
}
