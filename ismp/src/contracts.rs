//! Contains ismp primitives for contract support
use alloc::vec::Vec;
use codec::{Decode, Encode};

/// A return type that indicates the gas consumed by the contract executor
#[derive(PartialEq, Eq, Debug)]
pub struct Gas {
    /// Gas consumed when executing the contract call
    pub gas_used: Option<u64>,
    /// Gas limit passed to the contract executor
    pub gas_limit: Option<u64>,
}

impl From<()> for Gas {
    fn from(_: ()) -> Self {
        Self { gas_used: None, gas_limit: None }
    }
}

/// The contract data to provide additional metadata for the contract executor
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ContractData {
    /// Opaque bytes that would be decoded by the contract
    pub data: Vec<u8>,
    /// The gas limit the executor should use when executing the contract call
    pub gas_limit: u64,
}
