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

//! The ISMP request handler

use crate::{
    consensus_client::ConsensusClient,
    error::Error,
    handlers::{validate_state_machine, MessageResult, RequestResponseResult},
    host::ISMPHost,
    messaging::RequestMessage,
    router::RequestResponse,
};

/// Validate the state machine, verify the request message and dispatch the message to the router
pub fn handle_request_message(
    host: &dyn ISMPHost,
    msg: RequestMessage,
) -> Result<MessageResult, Error> {
    let consensus_client = validate_state_machine(host, &msg.proof)?;
    // Verify membership proof
    let state = host.state_machine_commitment(msg.proof.height)?;
    consensus_client.verify_membership(
        host,
        RequestResponse::Request(msg.request.clone()),
        state.commitment_root,
        &msg.proof,
    )?;

    let router = host.ismp_router();

    let result = RequestResponseResult {
        dest_chain: msg.request.dest_chain,
        source_chain: msg.request.source_chain,
        nonce: msg.request.nonce,
    };

    router.dispatch(msg.request)?;

    Ok(MessageResult::Request(result))
}
