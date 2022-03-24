pub mod error;
pub mod router;
pub mod safe;
pub mod utils;

pub type Result<T> = std::result::Result<T, error::Error>;
