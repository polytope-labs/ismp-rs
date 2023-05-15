// Copyright (C) Polytope Labs Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! ISMPRouter definition

use crate::{consensus::StateMachineHeight, host::StateMachine, prelude::Vec};
use alloc::string::String;
use codec::{Decode, Encode};
use core::time::Duration;
use primitive_types::{H160, H256, U256};

/// The ISMP POST request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Post {
    /// The source state machine of this request.
    pub source_chain: StateMachine,
    /// The destination state machine of this request.
    pub dest_chain: StateMachine,
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Moudle Id of the sending module
    pub from: Vec<u8>,
    /// Module ID of the receiving module
    pub to: Vec<u8>,
    /// Timestamp which this request expires in seconds.
    pub timeout_timestamp: u64,
    /// Encoded Request.
    pub data: Vec<u8>,
}

/// The ISMP GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Get {
    /// The source state machine of this request.
    pub source_chain: StateMachine,
    /// The destination state machine of this request.
    pub dest_chain: StateMachine,
    /// Storage kind
    pub storage_kind: StorageKind,
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Moudle Id of the sending module
    pub from: Vec<u8>,
    /// Storage keys that this request is interested in.
    pub keys: Vec<Vec<u8>>,
    /// Height at which to read the state machine.
    pub height: StateMachineHeight,
    /// Timestamp which this request expires in seconds
    pub timeout_timestamp: u64,
}

/// The ISMP request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum Request {
    /// A post request allows a module on a state machine to send arbitrary bytes to another module
    /// living in another state machine.
    Post(Post),
    /// A get request allows a module on a state machine to read the storage of another module
    /// living in another state machine.
    Get(Get),
}

/// The Storage Kind for GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum StorageKind {
    /// This indicates that we are trying to get an Evm storage
    EVM(EvmStorage),
    /// This indicates that we are trying to get a Substrate storage
    Substrate(SubstrateType),
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct EvmStorage {
    /// The contract address which is always 20 bytes
    pub contract_address: H160,
    /// To access contract storage, each variables are stored in a structure called Slot
    /// which is basically an increasing numerical index of the contract storage variables.
    /// We need to know the slot index of a variable before proceeding to query from the State
    /// trie.
    pub slot: u64,
    /// Different storage types supported by the EVM
    pub evm_storage_type: EvmStorageType,
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum EvmStorageType {
    /// An EVM Primitive value
    EvmPrimitive(EvmPrimitiveType),
    /// An EVM Array
    Array,
    /// An EVM Map
    Map,
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum EvmPrimitiveType {
    /// An EVM Address
    Address,
    /// An EVM uint8
    Uint8,
    /// An EVM uint32
    Uint32,
    /// An EVM uint64
    Uint64,
    /// An EVM uint128
    Uint128,
    /// An EVM uint256
    Uint256,
    /// An EVM boolean type
    Boolean,
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum SubstrateType {
    /// A Pallet
    Pallet(PalletStorageType),
    /// An Ink! smart contract
    Contract(InkContractStorage),
}

/// The Storage Type for Ink Get Request.
/// The storage API operates by storing and loading entries into and from a single storage cells,
/// where each storage cell is accessed under its own dedicated storage key.
/// Ink Storage data is always encoded with the SCALE codec.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct InkContractStorage {
    /// Account ID of the contract, which is a u32 byte
    pub account_id: H256,
    /// The contract's instantiation nonce
    pub instantiation_nonce: u64,
    /// Storage root key of the contract
    /// The storage key of the contracts root storage struct defaults to 0x00000000.
    /// However, contract developers can set the key to an arbitrary 4 bytes value by providing it
    /// a ManualKey
    pub storage_key: U256,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum PalletStorageType {
    /// Storage Value
    StorageValue,
    /// Storage Map
    StorageMap(PalletHasherType),
    /// Double Storage Map
    DoubleStorageMap(PalletHasherType),
    /// Storage N Map
    StorageNMap(PalletHasherType),
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum PalletHasherType {
    /// Blake 2
    Blake2_128Concat,
    /// TwoX
    TwoX64Concat,
    /// Identity
    Identity,
}

impl Request {
    /// Get the source chain
    pub fn source_chain(&self) -> StateMachine {
        match self {
            Request::Get(get) => get.source_chain,
            Request::Post(post) => post.source_chain,
        }
    }

    /// Get the destination chain
    pub fn dest_chain(&self) -> StateMachine {
        match self {
            Request::Get(get) => get.dest_chain,
            Request::Post(post) => post.dest_chain,
        }
    }

    /// Get the request nonce
    pub fn nonce(&self) -> u64 {
        match self {
            Request::Get(get) => get.nonce,
            Request::Post(post) => post.nonce,
        }
    }

    /// Get the POST request data
    pub fn data(&self) -> Option<Vec<u8>> {
        match self {
            Request::Get(_) => None,
            Request::Post(post) => Some(post.data.clone()),
        }
    }

    /// Get the GET request keys.
    pub fn keys(&self) -> Option<Vec<Vec<u8>>> {
        match self {
            Request::Post(_) => None,
            Request::Get(get) => Some(get.keys.clone()),
        }
    }

    /// Returns the timeout timestamp for a request
    pub fn timeout(&self) -> Duration {
        match self {
            Request::Post(post) => Duration::from_secs(post.timeout_timestamp),
            Request::Get(get) => Duration::from_secs(get.timeout_timestamp),
        }
    }

    /// Returns true if the destination chain timestamp has exceeded the request timeout timestamp
    pub fn timed_out(&self, proof_timestamp: Duration) -> bool {
        proof_timestamp >= self.timeout()
    }
}

/// The ISMP response
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Response {
    /// The request that triggered this response.
    pub request: Request,
    /// The response message.
    pub response: Vec<u8>,
}

/// This is the concrete type for Get requests
pub type GetResponse = Vec<(Vec<u8>, Vec<u8>)>;

/// Convenience enum for membership verification.
pub enum RequestResponse {
    Request(Vec<Request>),
    Response(Vec<Response>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DispatchSuccess {
    /// Destination chain for request or response
    pub dest_chain: StateMachine,
    /// Source chain for request or response
    pub source_chain: StateMachine,
    /// Request nonce
    pub nonce: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DispatchError {
    /// Descriptive error message
    pub msg: String,
    /// Request nonce
    pub nonce: u64,
    /// Source chain for request or response
    pub source: StateMachine,
    /// Destination chain for request or response
    pub dest: StateMachine,
}

pub type DispatchResult = Result<DispatchSuccess, DispatchError>;

pub trait ISMPRouter {
    /// Dispatch some requests to the ISMP router.
    /// For outgoing requests, they should be committed in state as a keccak256 hash
    /// For incoming requests, they should be dispatched to destination modules
    fn dispatch(&self, request: Request) -> DispatchResult;

    /// Dispatch request timeouts to the router which should dispatch them to modules
    fn dispatch_timeout(&self, request: Request) -> DispatchResult;

    /// Dispatch some responses to the ISMP router.
    /// For outgoing responses, the router should commit them to host state as a keccak256 hash
    /// For incoming responses, they should be dispatched to destination modules
    fn write_response(&self, response: Response) -> DispatchResult;
}
