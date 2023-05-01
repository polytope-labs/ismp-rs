extern crate alloc;
extern crate core;
// extern crate ismp;

use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, fmt::Debug, time::Duration};
use ismp::{
    consensus::ConsensusClient, error::Error, handlers::MessageResult, host::ISMPHost,
    messaging::ConsensusMessage,
};
use std::{collections::HashMap, time::SystemTime};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
