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

use crate::{consensus::StateMachineHeight, get::StorageKey, host::StateMachine, prelude::Vec};
use alloc::string::String;
use codec::{Decode, Encode};
use core::time::Duration;

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
    /// Module Id of the sending module
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
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Moudle Id of the sending module
    pub from: Vec<u8>,
    /// Storage keys that this request is interested in.
    pub keys: Vec<StorageKey>,
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
    pub fn keys(&self) -> Option<Vec<StorageKey>> {
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
