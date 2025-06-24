use core::fmt::Display;
use revm::context_interface::{
    result::{EVMError, InvalidTransaction},
    transaction::TransactionError,
};

/// Optimism transaction validation error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaikoTransactionError {
    Base(InvalidTransaction),
}

impl TransactionError for TaikoTransactionError {}

impl Display for TaikoTransactionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Base(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for TaikoTransactionError {}

impl From<InvalidTransaction> for TaikoTransactionError {
    fn from(value: InvalidTransaction) -> Self {
        Self::Base(value)
    }
}

impl<DBError> From<TaikoTransactionError> for EVMError<DBError, TaikoTransactionError> {
    fn from(value: TaikoTransactionError) -> Self {
        Self::Transaction(value)
    }
}
