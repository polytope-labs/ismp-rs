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

//! The ISMP request timeout handler

use crate::{
    error::Error,
    handlers::{validate_state_machine, MessageResult, RequestResponseResult},
    host::ISMPHost,
    messaging::TimeoutMessage,
};

/// This function handles timeouts for Requests
pub fn handle(host: &dyn ISMPHost, msg: TimeoutMessage) -> Result<MessageResult, Error> {
    let consensus_client = validate_state_machine(host, &msg.timeout_proof)?;
    let commitment = host.request_commitment(&msg.request)?;
    if commitment != host.get_request_commitment(&msg.request) {
        return Err(Error::RequestCommitmentNotFound {
            nonce: msg.request.nonce(),
            source: msg.request.source_chain(),
            dest: msg.request.dest_chain(),
        })
    }

    let now = host.timestamp();
    let state = host.state_machine_commitment(msg.timeout_proof.height)?;

    if now.as_secs() <= state.timestamp {
        Err(Error::RequestTimeoutVerificationFailed {
            nonce: msg.request.nonce(),
            source: msg.request.source_chain(),
            dest: msg.request.dest_chain(),
        })?
    }

    let key = host.get_request_commitment(&msg.request);

    consensus_client.verify_state_proof(host, key, state, &msg.timeout_proof)?;

    let router = host.ismp_router();
    router.dispatch_timeout(msg.request.clone())?;

    let result = RequestResponseResult {
        dest_chain: msg.request.source_chain(),
        source_chain: msg.request.dest_chain(),
        nonce: msg.request.nonce(),
    };

    Ok(MessageResult::Timeout(result))
}
