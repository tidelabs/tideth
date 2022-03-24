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
