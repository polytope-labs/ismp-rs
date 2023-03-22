#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
pub mod consensus_client;
pub mod error;
pub mod handlers;
pub mod host;
pub mod messaging;
pub mod module;
pub mod paths;
pub mod router;

pub mod prelude {
    pub use alloc::format;
    pub use alloc::str::FromStr;
    pub use alloc::string::String;
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
