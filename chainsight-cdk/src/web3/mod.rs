#[warn(clippy::module_inception)]
mod web3;
pub use web3::*;
pub mod abi;
pub use abi::*;
