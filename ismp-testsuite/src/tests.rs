use crate::{
    check_challenge_period, check_client_expiry, frozen_check,
    mocks::{Host, MockRouter},
    write_outgoing_commitments,
};
use ismp::{
    consensus::{IntermediateState, StateCommitment, StateMachineHeight, StateMachineId},
    host::StateMachine,
};
use std::sync::Arc;

#[test]
fn check_for_duplicate_requests_and_responses() {
    let host = Arc::new(Host::default());
    let router = MockRouter(host.clone());
    write_outgoing_commitments(&*host, &router).unwrap();
}

#[test]
fn should_reject_updates_within_challenge_period() {
    let host = Host::default();
    let id = [1u8; 4];
    check_challenge_period(
        &host,
        [1u8; 4],
        vec![],
        IntermediateState {
            height: StateMachineHeight {
                id: StateMachineId { state_id: StateMachine::Ethereum, consensus_client: id },
                height: 1,
            },
            commitment: StateCommitment {
                timestamp: 0,
                ismp_root: None,
                state_root: Default::default(),
            },
        },
        vec![],
    )
    .unwrap()
}

#[test]
fn should_reject_messages_for_frozen_state_machines() {
    let host = Host::default();
    let id = [1u8; 4];
    frozen_check(
        &host,
        [1u8; 4],
        vec![],
        IntermediateState {
            height: StateMachineHeight {
                id: StateMachineId { state_id: StateMachine::Ethereum, consensus_client: id },
                height: 1,
            },
            commitment: StateCommitment {
                timestamp: 0,
                ismp_root: None,
                state_root: Default::default(),
            },
        },
    )
    .unwrap()
}

#[test]
fn should_reject_expired_check_clients() {
    let host = Host::default();
    let id = [1u8; 4];
    check_client_expiry(&host, id, vec![], vec![]).unwrap()
}
