use std::{
    cell::RefCell,
    collections::HashMap,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use codec::Encode;

use crate::{
    consensus_client::{
        ConsensusClientId, IntermediateState, StateCommitment, StateMachineHeight, StateMachineId,
    },
    error::Error,
    handlers::handle_incoming_message,
    host::{ChainID, ISMPHost},
    messaging::{ConsensusMessage, Message},
    mock::{Host, Polkadot, Stage, POLKADOT_CONSENSUS_CLIENT_ID},
    router::{Request, POST},
};

#[test]
fn request_commitment_retrieves_works() {
    let moonbeam = Host {
        commits: RefCell::new(HashMap::<StateMachineHeight, StateCommitment>::new()),
        req_commit: RefCell::new(HashMap::<Request, StateCommitment>::new()),
        router: Stage,
        frozen: RefCell::new(Vec::<StateMachineHeight>::new()),
        client: RefCell::new(HashMap::<ConsensusClientId, Polkadot>::new()),
        latest_height: RefCell::new(HashMap::<StateMachineId, StateMachineHeight>::new()),
    };

    let id0 = StateMachineId { state_id: 0, consensus_client: 0 };

    // Initial Polkadot consensus client
    let entry0 = IntermediateState {
        height: StateMachineHeight { id: id0, height: 0 },
        commitment: StateCommitment {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            ismp_root: [0; 32],
            state_root: [0; 32],
        },
    };

    let mut polkadot_state: Vec<IntermediateState> = Vec::new();

    polkadot_state.push(entry0);

    let encoded_polkadot_state = (polkadot_state, false, Duration::from_secs(60)).encode();

    moonbeam.client.borrow_mut().insert(
        POLKADOT_CONSENSUS_CLIENT_ID,
        Polkadot {
            id: POLKADOT_CONSENSUS_CLIENT_ID,
            state: encoded_polkadot_state,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        },
    );

    let req = Request::Post(POST {
        nonce: 0,
        source_chain: ChainID::HYPERSPACE,
        dest_chain: ChainID::MOONBEAM,
        from: Vec::new(),
        to: Vec::new(),
        timeout_timestamp: 0,
        data: Vec::new(),
    });

    let id1 = StateMachineId { state_id: 1, consensus_client: POLKADOT_CONSENSUS_CLIENT_ID };

    let height1 = StateMachineHeight { id: id1, height: 100 };

    let commit1 = StateCommitment {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        ismp_root: [0; 32],
        state_root: [0; 32],
    };

    moonbeam.commits.borrow_mut().insert(height1, commit1.clone());

    moonbeam.req_commit.borrow_mut().insert(req.clone(), commit1);

    assert_eq!(moonbeam.request_commitment(&req), Ok([0; 32].to_vec()));
}

#[test]
fn bounce_frozen_state_messages() {
    let astar = Host {
        commits: RefCell::new(HashMap::<StateMachineHeight, StateCommitment>::new()),
        req_commit: RefCell::new(HashMap::<Request, StateCommitment>::new()),
        router: Stage,
        frozen: RefCell::new(Vec::<StateMachineHeight>::new()),
        client: RefCell::new(HashMap::<ConsensusClientId, Polkadot>::new()),
        latest_height: RefCell::new(HashMap::<StateMachineId, StateMachineHeight>::new()),
    };

    let id0 = StateMachineId { state_id: 0, consensus_client: 0 };

    // Initial Polkadot consensus client
    let entry0 = IntermediateState {
        height: StateMachineHeight { id: id0, height: 0 },
        commitment: StateCommitment {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            ismp_root: [0; 32],
            state_root: [0; 32],
        },
    };

    let mut polkadot_state: Vec<IntermediateState> = Vec::new();

    polkadot_state.push(entry0);

    let encoded_polkadot_state = (polkadot_state, false, Duration::from_secs(60)).encode();

    astar.client.borrow_mut().insert(
        POLKADOT_CONSENSUS_CLIENT_ID,
        Polkadot {
            id: POLKADOT_CONSENSUS_CLIENT_ID,
            state: encoded_polkadot_state,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        },
    );

    let mut consensus_proof: Vec<IntermediateState> = Vec::new();

    let id1 = StateMachineId { state_id: 1, consensus_client: 1 };

    let id2 = StateMachineId { state_id: 2, consensus_client: 2 };

    let height1 = StateMachineHeight { id: id1, height: 100 };

    let height2 = StateMachineHeight { id: id2, height: 200 };

    astar.freeze_state_machine(height2).unwrap();

    let commit1 = StateCommitment { timestamp: 123456789, ismp_root: [0; 32], state_root: [0; 32] };

    let entry1 = IntermediateState { height: height1, commitment: commit1.clone() };

    consensus_proof.push(entry1);

    let entry2 = IntermediateState {
        height: height2,
        commitment: StateCommitment {
            timestamp: 234567890,
            ismp_root: [0; 32],
            state_root: [0; 32],
        },
    };

    consensus_proof.push(entry2);

    let encoded_consensus_proof = consensus_proof.encode();

    thread::sleep(Duration::from_secs(60));
    handle_incoming_message(
        &astar,
        Message::Consensus(ConsensusMessage {
            consensus_proof: encoded_consensus_proof,
            consensus_client_id: POLKADOT_CONSENSUS_CLIENT_ID,
        }),
    )
    .unwrap();

    assert_eq!(astar.state_machine_commitment(height1), Ok(commit1));

    assert_eq!(
        astar.state_machine_commitment(height2),
        Err(Error::StateCommitmentNotFound { height: height2 })
    );
}
