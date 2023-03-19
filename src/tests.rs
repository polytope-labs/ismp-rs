use core::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::{mock::{HP, Stage, Ethereum}, consensus_client::{StateMachineHeight, StateCommitment, StateMachineId, ConsensusClientId, ETHEREUM_CONSENSUS_CLIENT_ID}, router::{Request, Response}, handlers::handle_incoming_message, messaging::{Message, ConsensusMessage}};

#[test]
fn handle_incoming_messages_works() {
    let mut new_host = HP {
        state: RefCell::new(HashMap::<StateMachineId, StateMachineHeight>::new()),
        consensus: RefCell::new(HashMap::<ConsensusClientId, StateCommitment>::new()),
        commits: RefCell::new(HashMap::<StateMachineHeight, StateCommitment>::new()),
        req_commit: RefCell::new(HashMap::<Request, StateCommitment>::new()),
        res_commit: RefCell::new(HashMap::<Response, StateCommitment>::new()),
        timestamp: Duration::new(5, 0),
        router: Stage,
        client: RefCell::new(HashMap::<ConsensusClientId, Ethereum>::new()),
        mc: RefCell::new(HashMap::<StateMachineId, ConsensusClientId>::new())
    };

    handle_incoming_message(&mut new_host, Message::Consensus(ConsensusMessage{ consensus_proof: Vec::<u8>::new(), consensus_client_id: ETHEREUM_CONSENSUS_CLIENT_ID})).ok();
}