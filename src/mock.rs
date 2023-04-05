use codec::{Decode, Encode};
use core::borrow::Borrow;
use std::alloc::System;
use std::collections::HashMap;
use std::process::id;

use crate::consensus_client::{
    ConsensusClient, ConsensusClientId, IntermediateState, StateMachineHeight, StateMachineId,
};
use crate::{
    consensus_client::StateCommitment,
    error::Error,
    host,
    router::IISMPRouter,
    router::{Request, Response},
};

use std::cell::RefCell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const POLKADOT_CONSENSUS_CLIENT_ID: ConsensusClientId = 300;

#[derive(Debug)]
pub struct HP {
    pub commits: RefCell<HashMap<StateMachineHeight, StateCommitment>>,
    pub req_commit: RefCell<HashMap<Request, StateCommitment>>,
    pub router: Stage,
    pub frozen: RefCell<Vec<StateMachineHeight>>,
    pub client: RefCell<HashMap<ConsensusClientId, Polkadot>>,
    pub latest_height: RefCell<HashMap<StateMachineId, StateMachineHeight>>,
}

impl host::ISMPHost for HP {
    fn host(&self) -> host::ChainID {
        host::ChainID::HYPERSPACE
    }

    fn latest_commitment_height(&self, id: StateMachineId) -> Result<StateMachineHeight, Error> {
        if let Some(height) = self.latest_height.borrow().get(&id) {
            return Ok(*height);
        } else {
            Ok(StateMachineHeight {
                id: StateMachineId {
                    state_id: 0,
                    consensus_client: 0,
                },
                height: 0,
            })
        }
    }

    fn state_machine_commitment(
        &self,
        height: StateMachineHeight,
    ) -> Result<StateCommitment, Error> {
        if let Some(commit) = self.commits.borrow().get(&height) {
            return Ok(commit.clone());
        } else {
            Err(Error::StateCommitmentNotFound { height })
        }
    }

    fn consensus_update_time(&self, _id: ConsensusClientId) -> Result<Duration, Error> {
        todo!()
    }

    fn consensus_state(&self, id: ConsensusClientId) -> Result<Vec<u8>, Error> {
        if let Some(state) = self.client.borrow().get(&id) {
            return Ok(state.state.clone());
        } else {
            Err(Error::ConsensusStateNotFound { id })
        }
    }

    fn host_timestamp(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime::duration_since failed")
    }

    fn is_frozen(&self, height: StateMachineHeight) -> Result<bool, Error> {
        if self.frozen.borrow().contains(&height) {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn request_commitment(&self, req: &Request) -> Result<Vec<u8>, Error> {
        if let Some(commit) = self.req_commit.borrow().get(&req) {
            return Ok(commit.commitment_root.clone());
        } else {
            return Err(Error::RequestCommitmentNotFound {
                nonce: req.nonce,
                source: req.source_chain,
                dest: req.dest_chain,
            });
        }
    }

    fn store_consensus_state(&self, id: ConsensusClientId, state: Vec<u8>) -> Result<(), Error> {
        self.client
            .borrow_mut()
            .get_mut(&id)
            .map(|old_state| old_state.state = state)
            .unwrap();
        Ok(())
    }

    fn store_consensus_update_time(
        &self,
        id: ConsensusClientId,
        timestamp: Duration,
    ) -> Result<(), Error> {
        self.client
            .borrow_mut()
            .get_mut(&id)
            .map(|client| client.timestamp = timestamp.as_secs())
            .unwrap();
        Ok(())
    }

    fn store_state_machine_commitment(
        &self,
        height: StateMachineHeight,
        state: StateCommitment,
    ) -> Result<(), Error> {
        let mut borrowed = self.commits.borrow_mut();
        if let Some(commit) = borrowed.get_mut(&height) {
            *commit = state;
            Ok(())
        } else {
            borrowed.insert(height, state);
            Ok(())
        }
    }

    fn freeze_state_machine(&self, height: StateMachineHeight) -> Result<(), Error> {
        let mut state_machine_heights = self.frozen.borrow_mut();
        if !state_machine_heights.contains(&height) {
            state_machine_heights.push(height);
        }
        Ok(())
    }

    fn store_latest_commitment_height(&self, height: StateMachineHeight) -> Result<(), Error> {
        let mut borrowed = self.latest_height.borrow_mut();
        if let Some(latest) = borrowed.get_mut(&height.id) {
            *latest = height;
            Ok(())
        } else {
            borrowed.insert(height.id, height);
            Ok(())
        }
    }

    fn consensus_client(&self, id: ConsensusClientId) -> Result<Box<dyn ConsensusClient>, Error> {
        match self
            .client
            .borrow()
            .get(&id)
            .ok_or(Error::ImplementationSpecific(
                "Client does not exist".to_string(),
            ))
            .cloned()
        {
            Ok(val) => Ok(Box::new(val)),
            Err(e) => Err(e),
        }
    }

    fn keccak256(&self, _bytes: &[u8]) -> [u8; 32] {
        todo!()
    }

    fn delay_period(&self, _id: u64) -> Duration {
        Duration::from_secs(10 * 60)
    }

    fn ismp_router(&self) -> Box<dyn IISMPRouter> {
        Box::new(self.router.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Polkadot {
    pub(crate) state: Vec<u8>, // could the state also contain a boolean(frozen state)
    pub(crate) timestamp: u64,
}
impl ConsensusClient for Polkadot {
    fn verify(
        &self,
        host: &dyn host::ISMPHost,
        trusted_consensus_state: Vec<u8>,
        proof: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<IntermediateState>), Error> {
        let states: Vec<IntermediateState> =
            Decode::decode(&mut trusted_consensus_state.as_ref()).unwrap();
        let proofs: Vec<IntermediateState> = Decode::decode(&mut proof.as_ref()).unwrap();
        let mut uncommited_state: Vec<IntermediateState> = Vec::new();

        // if the key already exists in the statemachine commit then discard it from the consensus client.
        for state in states {
            if !host.state_machine_commitment(state.height).is_ok() {
                uncommited_state.push(state);
            }
        }
        uncommited_state.extend(proofs);
        let encoded: Vec<u8> = uncommited_state.encode();
        return Ok((encoded, uncommited_state));
    }

    fn consensus_id(&self) -> ConsensusClientId {
        POLKADOT_CONSENSUS_CLIENT_ID
    }

    fn unbonding_period(&self) -> Duration {
        todo!()
    }

    fn verify_membership(
        &self,
        host: &dyn host::ISMPHost,
        key: Vec<u8>,
        commitment: Vec<u8>,
        proof: &crate::messaging::Proof,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_non_membership(
        &self,
        host: &dyn host::ISMPHost,
        key: Vec<u8>,
        commitment: Vec<u8>,
        proof: &crate::messaging::Proof,
    ) -> Result<(), Error> {
        todo!()
    }

    fn is_frozen(&self, host: &dyn host::ISMPHost, id: ConsensusClientId) -> Result<bool, Error> {
        /*
        match host.consensus_state(id) {
            Ok(mut state) => {
                let (_, decoded_bool): (Vec<IntermediateState>, bool) =
                    Decode::decode(&mut state.as_slice()).unwrap();
                return Ok(decoded_bool);
            }
            Err(e) => Err(e),
        }
        */
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Stage;
impl IISMPRouter for Stage {
    fn dispatch(&self, request: Request) -> Result<(), Error> {
        todo!()
    }

    fn write_response(&self, response: Response) -> Result<(), Error> {
        todo!()
    }
}
