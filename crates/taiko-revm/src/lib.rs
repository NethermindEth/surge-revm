//! Taiko-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod api;
pub mod evm;
pub mod handler;
pub mod l1block;
pub mod precompiles;
pub mod result;
pub mod spec;
pub mod transaction;

pub use api::{
    builder::TaikoBuilder,
    default_ctx::{DefaultTaiko, TaikoContext},
};
pub use evm::TaikoEvm;
pub use l1block::L1BlockInfo;
pub use result::TaikoHaltReason;
pub use spec::*;
pub use transaction::{error::TaikoTransactionError, TaikoTransaction};
