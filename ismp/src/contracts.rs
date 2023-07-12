//! Contains ismp primitives for contract support

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
