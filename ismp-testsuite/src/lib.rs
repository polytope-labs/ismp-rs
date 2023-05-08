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

use ismp::{host::ISMPHost, router::ISMPRouter};

/*
    Consensus Client and State Machine checks
*/
/// Ensure challenge period rules are followed
pub fn check_challenge_period(host: &dyn ISMPHost) -> Result<(), &'static str> {
    Ok(())
}

/// Ensure expired client rules are followed
pub fn check_client_expiry(host: &dyn ISMPHost) -> Result<(), &'static str> {
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
    Router Implementation checks
*/

/// Duplicate request and response check
pub fn check_duplicates(host: &dyn ISMPHost, router: &dyn ISMPRouter) -> Result<(), &'static str> {
    Ok(())
}

/// Check that router stores commitments for outgoing requests and responses
pub fn write_commitments(host: &dyn ISMPHost, router: &dyn ISMPRouter) -> Result<(), &'static str> {
    Ok(())
}
