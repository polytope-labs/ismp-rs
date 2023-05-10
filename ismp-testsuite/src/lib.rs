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

//! ISMP Testsuite

mod mocks;
#[cfg(test)]
mod tests;

use ismp::{
    consensus::ConsensusClientId,
    handlers::handle_incoming_message,
    host::{ISMPHost, StateMachine},
    messaging::{ConsensusMessage, Message},
    router::{ISMPRouter, Post, Request, Response},
};

/*
    Consensus Client and State Machine checks
*/
/// Ensure challenge period rules are followed
pub fn check_challenge_period<H: ISMPHost>(
    host: &H,
    id: ConsensusClientId,
    cs_state: Vec<u8>,
    consensus_proof: Vec<u8>,
) -> Result<(), &'static str> {
    let consensus_message =
        Message::Consensus(ConsensusMessage { consensus_proof, consensus_client_id: id });
    host.store_consensus_state(id, cs_state).unwrap();

    // Set the previous update time
    let challenge_period = host.challenge_period(id);
    let previous_update_time = host.timestamp() - (challenge_period / 2);
    host.store_consensus_update_time(id, previous_update_time).unwrap();

    let res = handle_incoming_message::<H>(host, consensus_message);
    assert!(matches!(res, Err(ismp::error::Error::ChallengePeriodNotElapsed { .. })));
    Ok(())
}

/// Ensure expired client rules are followed
pub fn check_client_expiry<H: ISMPHost>(
    host: &H,
    id: ConsensusClientId,
    cs_state: Vec<u8>,
    consensus_proof: Vec<u8>,
) -> Result<(), &'static str> {
    let consensus_message =
        Message::Consensus(ConsensusMessage { consensus_proof, consensus_client_id: id });
    host.store_consensus_state(id, cs_state).unwrap();
    // Set the previous update time
    let client = host.consensus_client(id).unwrap();
    let unbonding_period = client.unbonding_period();
    let previous_update_time = host.timestamp() - unbonding_period;
    host.store_consensus_update_time(id, previous_update_time).unwrap();

    let res = handle_incoming_message::<H>(host, consensus_message);

    assert!(matches!(res, Err(ismp::error::Error::UnbondingPeriodElapsed { .. })));
    Ok(())
}

/// Frozen client and state machine checks
pub fn frozen_check(host: &dyn ISMPHost) -> Result<(), &'static str> {
    Ok(())
}

/// Ensure all timeout post processing is correctly done.
pub fn timeout_post_processing_check(
    host: &dyn ISMPHost,
    router: &dyn ISMPRouter,
) -> Result<(), &'static str> {
    Ok(())
}

/*
    Check correctness of router implementation
*/

/// Check that router stores commitments for outgoing requests and responses and rejects duplicates
pub fn write_outgoing_commitments(
    host: &dyn ISMPHost,
    router: &dyn ISMPRouter,
) -> Result<(), &'static str> {
    let post = Post {
        source_chain: host.host_state_machine(),
        dest_chain: StateMachine::Kusama(2000),
        nonce: 0,
        from: vec![0u8; 32],
        to: vec![0u8; 32],
        timeout_timestamp: 0,
        data: vec![0u8; 64],
    };
    let request = Request::Post(post);
    // Dispatch the request the first time
    router.dispatch(request.clone()).map_err(|_| "Router failed to dispatch request")?;
    // Fetch commitment from storage
    host.request_commitment(&request)
        .map_err(|_| "Expected Request commitment to be found in storage")?;
    // Dispatch the same request a second time
    let err = router.dispatch(request.clone());
    assert!(err.is_err(), "Expected router to return error for duplicate request");
    let post = Post {
        source_chain: StateMachine::Kusama(2000),
        dest_chain: host.host_state_machine(),
        nonce: 0,
        from: vec![0u8; 32],
        to: vec![0u8; 32],
        timeout_timestamp: 0,
        data: vec![0u8; 64],
    };
    let response = Response { request: Request::Post(post), response: vec![0u8; 64] };
    // Dispatch the outgoing response for the first time
    router.write_response(response.clone()).map_err(|_| "Router failed to dispatch request")?;
    // Dispatch the same response a second time
    let err = router.write_response(response);
    assert!(err.is_err(), "Expected router to return error for duplicate response");

    Ok(())
}
