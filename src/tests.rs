use std::cell::RefCell;
use std::collections::HashMap;

use codec::Encode;

use crate::{
    consensus_client::{
        ConsensusClientId, IntermediateState, StateCommitment, StateMachineHeight, StateMachineId,
    },
    error::Error,
    handlers::handle_incoming_message,
    host::{ChainID, ISMPHost},
    messaging::{ConsensusMessage, Message},
    mock::{Polkadot, Stage, HP, POLKADOT_CONSENSUS_CLIENT_ID},
    router::Request,
};

#[test]
fn request_commitment_retrieves_works() {
    let new_host = HP {
        commits: RefCell::new(HashMap::<StateMachineHeight, StateCommitment>::new()),
        req_commit: RefCell::new(HashMap::<Request, StateCommitment>::new()),
        router: Stage,
        frozen: RefCell::new(Vec::<StateMachineHeight>::new()),
        client: RefCell::new(HashMap::<ConsensusClientId, Polkadot>::new()),
        latest_height: RefCell::new(HashMap::<StateMachineId, StateMachineHeight>::new()),
    };

    // Initialize req_commit
    let req = Request {
        nonce: 0,
        source_chain: ChainID::MOONBEAM,
        dest_chain: ChainID::HYPERSPACE,
        from: Vec::new(),
        to: Vec::new(),
        timeout_timestamp: 0,
        data: Vec::new(),
    };
    new_host.req_commit.borrow_mut().insert(
        req.clone(),
        StateCommitment {
            timestamp: 1,
            commitment_root: Vec::new(),
        },
    );

    assert_eq!(new_host.request_commitment(&req), Ok(Vec::new()));
}

#[test]
fn bounce_frozen_state_messages() {
    let new_host = HP {
        commits: RefCell::new(HashMap::<StateMachineHeight, StateCommitment>::new()),
        req_commit: RefCell::new(HashMap::<Request, StateCommitment>::new()),
        router: Stage,
        frozen: RefCell::new(Vec::<StateMachineHeight>::new()),
        client: RefCell::new(HashMap::<ConsensusClientId, Polkadot>::new()),
        latest_height: RefCell::new(HashMap::<StateMachineId, StateMachineHeight>::new()),
    };

    let mut consensus_proof: Vec<IntermediateState> = Vec::new();

    let id0 = StateMachineId {
        state_id: 0,
        consensus_client: 0,
    };

    let id1 = StateMachineId {
        state_id: 1,
        consensus_client: 1,
    };

    let id2 = StateMachineId {
        state_id: 2,
        consensus_client: 2,
    };

    let frozen_height = StateMachineHeight {
        id: id2,
        height: 200,
    };

    let height1 = StateMachineHeight {
        id: id1,
        height: 100,
    };

    let commit1 = StateCommitment {
        timestamp: 123456789,
        commitment_root: vec![0x12, 0x34, 0x56],
    };

    let entry1 = IntermediateState {
        height: height1,
        commitment: commit1.clone(),
    };

    let entry2 = IntermediateState {
        height: frozen_height,
        commitment: StateCommitment {
            timestamp: 234567890,
            commitment_root: vec![0xab, 0xcd, 0xef],
        },
    };

    consensus_proof.push(entry1);
    consensus_proof.push(entry2);

    let encoded_consensus_proof = consensus_proof.encode();

    // Polkadot consensus state
    let entry0 = IntermediateState {
        height: StateMachineHeight { id: id0, height: 0 },
        commitment: StateCommitment {
            timestamp: 123455789,
            commitment_root: vec![0xff, 0xaa, 0x55],
        },
    };

    let mut polkadot_state: Vec<IntermediateState> = Vec::new();
    polkadot_state.push(entry0);
    let encoded_polkadot_state = polkadot_state.encode();

    new_host.client.borrow_mut().insert(
        POLKADOT_CONSENSUS_CLIENT_ID,
        Polkadot {
            state: encoded_polkadot_state,
            timestamp: 0,
        },
    );
    new_host.frozen.borrow_mut().push(frozen_height);

    handle_incoming_message(
        &new_host,
        Message::Consensus(ConsensusMessage {
            consensus_proof: encoded_consensus_proof,
            consensus_client_id: POLKADOT_CONSENSUS_CLIENT_ID,
        }),
    )
    .unwrap();

    assert_eq!(new_host.state_machine_commitment(height1), Ok(commit1));

    assert_eq!(
        new_host.state_machine_commitment(frozen_height),
        Err(Error::StateCommitmentNotFound {
            height: frozen_height
        })
    );
}
