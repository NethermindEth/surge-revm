use auto_impl::auto_impl;
use revm::{
    context::TxEnv,
    context_interface::transaction::Transaction,
    primitives::{Address, Bytes, TxKind, B256, U256},
};

#[auto_impl(&, &mut, Box, Arc)]
pub trait TaikoTxTr: Transaction {
    /// Retnruns the treasury address for the transaction
    fn treasury(&self) -> Address;
    /// Returns the base fee ration
    fn basefee_ratio(&self) -> u8;
    /// Returns `true` if the transaction is anchor transaction
    fn is_anchor(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaikoTransaction<T: Transaction> {
    /// The enveloped transaction, if any.
    pub base: T,
    /// Treasury address for the transaction.
    pub treasury: Address,
    /// Base fee ratio for the transaction.
    pub basefee_ratio: u8,
    /// If the transaction is anchor transaction.
    pub is_anchor: bool,
}

impl Default for TaikoTransaction<TxEnv> {
    fn default() -> Self {
        Self {
            base: TxEnv::default(),
            treasury: Address::ZERO,
            basefee_ratio: 0,
            is_anchor: false,
        }
    }
}

impl<T: Transaction> Transaction for TaikoTransaction<T> {
    type AccessListItem<'a>
        = T::AccessListItem<'a>
    where
        T: 'a;
    type Authorization<'a>
        = T::Authorization<'a>
    where
        T: 'a;

    fn tx_type(&self) -> u8 {
        self.base.tx_type()
    }

    fn caller(&self) -> Address {
        self.base.caller()
    }

    fn gas_limit(&self) -> u64 {
        self.base.gas_limit()
    }

    fn value(&self) -> U256 {
        self.base.value()
    }

    fn input(&self) -> &Bytes {
        self.base.input()
    }

    fn nonce(&self) -> u64 {
        self.base.nonce()
    }

    fn kind(&self) -> TxKind {
        self.base.kind()
    }

    fn chain_id(&self) -> Option<u64> {
        self.base.chain_id()
    }

    fn access_list(&self) -> Option<impl Iterator<Item = Self::AccessListItem<'_>>> {
        self.base.access_list()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.base.max_priority_fee_per_gas()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.base.max_fee_per_gas()
    }

    fn gas_price(&self) -> u128 {
        self.base.gas_price()
    }

    fn blob_versioned_hashes(&self) -> &[B256] {
        self.base.blob_versioned_hashes()
    }

    fn max_fee_per_blob_gas(&self) -> u128 {
        self.base.max_fee_per_blob_gas()
    }

    fn effective_gas_price(&self, base_fee: u128) -> u128 {
        self.base.effective_gas_price(base_fee)
    }

    fn authorization_list_len(&self) -> usize {
        self.base.authorization_list_len()
    }

    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>> {
        self.base.authorization_list()
    }

    // TODO(EOF)
    // fn initcodes(&self) -> &[Bytes] {
    //     self.base.initcodes()
    // }
}

impl<T: Transaction> TaikoTxTr for TaikoTransaction<T> {
    fn treasury(&self) -> Address {
        self.treasury
    }

    fn basefee_ratio(&self) -> u8 {
        self.basefee_ratio
    }

    fn is_anchor(&self) -> bool {
        self.is_anchor
    }
}
