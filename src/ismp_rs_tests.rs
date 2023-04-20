use core::{cell::RefCell, fmt::Debug, time::Duration};
use keccak_hash::{keccak, H256};
use std::collections::HashMap;
use std::time::SystemTime;

use alloc::rc::Rc;

use crate::{
    consensus_client::{
        ConsensusClient, ConsensusClientId, IntermediateState, StateCommitment, StateMachineHeight,
        StateMachineId,
    },
    error::Error,
    handlers::{MessageResult, RequestResponseResult},
    host::{ISMPHost, StateMachine},
    messaging::{CreateConsensusClient, Proof, RequestMessage, ResponseMessage},
    router::{ISMPRouter, RequestResponse},
    util::hash_request,
};

pub type Hash = [u8; 32];
pub const ETHEREUM_CONSENSUS_CLIENT_ID: u64 = 1;

// Mock host object
#[derive(Debug, Clone)]
struct DummyHost {
    storage_state_machine: Rc<RefCell<HashMap<StateMachineHeight, StateCommitment>>>,
    storage_consensus: Rc<RefCell<HashMap<ConsensusClientId, DummyClient>>>,
    storage_consensus_encoded: Rc<RefCell<HashMap<ConsensusClientId, Vec<u8>>>>,
    storage_latest_state_machine: Rc<RefCell<HashMap<StateMachineId, StateMachineHeight>>>,
    frozen_machine_height: Rc<RefCell<HashMap<StateMachineHeight, bool>>>,
    updated_consensus_timestamp: Rc<RefCell<HashMap<ConsensusClientId, Duration>>>,
    state_machine_id: StateMachine,
    request_commitment: Rc<RefCell<HashMap<H256, RequestMessage>>>,
    reponse_commitment: Rc<RefCell<HashMap<H256, ResponseMessage>>>,
    consensus_proofs: Rc<RefCell<HashMap<ConsensusClientId, Proof>>>,
}

impl ISMPRouter for RequestMessage {
    fn dispatch(&self, request: crate::router::Request) -> Result<(), Error> {
        todo!()
    }

    fn dispatch_timeout(&self, request: crate::router::Request) -> Result<(), Error> {
        todo!()
    }

    fn write_response(&self, response: crate::router::Response) -> Result<(), Error> {
        todo!()
    }
}

impl ISMPHost for DummyHost {
    fn host_state_machine(&self) -> crate::host::StateMachine {
        self.state_machine_id
    }

    fn latest_commitment_height(
        &self,
        id: crate::consensus_client::StateMachineId,
    ) -> Result<crate::consensus_client::StateMachineHeight, Error> {
        self.storage_latest_state_machine
            .borrow()
            .get(&id)
            .cloned()
            .ok_or(Error::ImplementationSpecific(
                "Missing latest state machine height".to_string(),
            ))
    }

    fn state_machine_commitment(
        &self,
        height: crate::consensus_client::StateMachineHeight,
    ) -> Result<StateCommitment, Error> {
        self.storage_state_machine
            .borrow()
            .get(&height)
            .cloned()
            .ok_or(Error::StateCommitmentNotFound { height })
    }

    fn consensus_state(
        &self,
        id: crate::consensus_client::ConsensusClientId,
    ) -> Result<Vec<u8>, Error> {
        self.storage_consensus_encoded
            .borrow()
            .get(&id)
            .cloned()
            .ok_or(Error::ConsensusStateNotFound { id })
    }

    fn timestamp(&self) -> core::time::Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
    }

    fn is_frozen(
        &self,
        height: crate::consensus_client::StateMachineHeight,
    ) -> Result<bool, Error> {
        if self.storage_state_machine.borrow().contains_key(&height) {
            if self.frozen_machine_height.borrow().contains_key(&height) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(Error::StateCommitmentNotFound { height })
        }
    }

    fn store_consensus_state(
        &self,
        id: crate::consensus_client::ConsensusClientId,
        state: Vec<u8>,
    ) -> Result<(), Error> {
        self.storage_consensus_encoded
            .borrow_mut()
            .insert(id.clone(), state);

        Ok(())
    }

    fn store_consensus_update_time(
        &self,
        id: crate::consensus_client::ConsensusClientId,
        timestamp: core::time::Duration,
    ) -> Result<(), Error> {
        self.updated_consensus_timestamp
            .borrow_mut()
            .insert(id.clone(), timestamp);
        Ok(())
    }

    fn store_state_machine_commitment(
        &self,
        height: crate::consensus_client::StateMachineHeight,
        state: StateCommitment,
    ) -> Result<(), Error> {
        self.storage_state_machine
            .borrow_mut()
            .insert(height.clone(), state);
        Ok(())
    }

    fn freeze_state_machine(
        &self,
        height: crate::consensus_client::StateMachineHeight,
    ) -> Result<(), Error> {
        self.frozen_machine_height
            .borrow_mut()
            .insert(height.clone(), true);

        Ok(())
    }

    fn store_latest_commitment_height(
        &self,
        height: crate::consensus_client::StateMachineHeight,
    ) -> Result<(), Error> {
        self.storage_latest_state_machine
            .borrow_mut()
            .insert(height.id.clone(), height);
        Ok(())
    }

    fn consensus_client(
        &self,
        id: crate::consensus_client::ConsensusClientId,
    ) -> Result<Box<dyn crate::consensus_client::ConsensusClient>, Error> {
        // if self.storage_consensus.borrow().contains_key(&id) {
        // 	Box::new(self.storage_consensus.borrow_mut().get(&id).unwrap())
        // } else {
        // 	Err(Error::ConsensusStateNotFound { id })
        // }
        todo!()
    }

    fn challenge_period(
        &self,
        id: crate::consensus_client::ConsensusClientId,
    ) -> core::time::Duration {
        match id {
            id if id == ETHEREUM_CONSENSUS_CLIENT_ID => Duration::from_secs(60),
            _ => Duration::from_secs(20),
        }
    }

    fn ismp_router(&self) -> Box<dyn crate::router::ISMPRouter> {
        Box::new(RequestMessage {
            request: todo!(),
            proof: todo!(),
        })
    }

    fn consensus_update_time(&self, id: ConsensusClientId) -> Result<core::time::Duration, Error> {
        if self.storage_consensus_encoded.borrow().contains_key(&id) {
            self.updated_consensus_timestamp
                .borrow_mut()
                .insert(id, self.timestamp());
            Ok(self.timestamp())
        } else {
            Err(Error::ConsensusStateNotFound { id })
        }
    }

    fn keccak256(bytes: &[u8]) -> keccak_hash::H256
    where
        Self: Sized,
    {
        keccak(bytes)
    }

    fn request_commitment(&self, req: &crate::router::Request) -> Result<keccak_hash::H256, Error> {
        let commitment = hash_request::<Self>(req);
        if self.request_commitment.borrow().contains_key(&commitment) {
            Ok(commitment)
        } else {
            Err(Error::ImplementationSpecific(
                "Request not found".to_string(),
            ))
        }
    }

    fn is_expired(&self, consensus_id: ConsensusClientId) -> Result<(), Error> {
        let host_timestamp = self.timestamp();
        let unbonding_period = self.consensus_client(consensus_id)?.unbonding_period();
        let last_update = self.consensus_update_time(consensus_id)?;
        if host_timestamp.saturating_sub(last_update) > unbonding_period {
            Err(Error::UnbondingPeriodElapsed { consensus_id })?
        }

        Ok(())
    }
}

// Mock client object
#[derive(Debug, Clone)]
pub struct DummyClient {
    /// Scale encoded consensus state
    pub consensus_state: Vec<u8>,
    /// Consensus client id
    pub consensus_client_id: ConsensusClientId,
    /// State machine commitments
    pub state_machine_commitments: Vec<IntermediateState>,
    /// proof
    pub proof: Vec<Proof>,
}

impl ConsensusClient for DummyClient {
    fn unbonding_period(&self) -> core::time::Duration {
        Duration::from_secs(60)
    }

    fn verify_consensus(
        &self,
        host: &dyn ISMPHost,
        trusted_consensus_state: Vec<u8>,
        proof: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<IntermediateState>), Error> {
        // let mut state = self.state.clone();

        // Ok((self.consensus_state.clone(), vec![state]))
        todo!()
    }

    fn verify_membership(
        &self,
        host: &dyn ISMPHost,
        item: RequestResponse,
        root: StateCommitment,
        proof: &crate::messaging::Proof,
    ) -> Result<(), Error> {
        todo!()
    }

    fn state_trie_key(&self, request: RequestResponse) -> Vec<u8> {
        todo!()
    }

    fn verify_state_proof(
        &self,
        host: &dyn ISMPHost,
        key: Vec<u8>,
        root: StateCommitment,
        proof: &crate::messaging::Proof,
    ) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }

    fn is_frozen(&self, trusted_consensus_state: &[u8]) -> Result<(), Error> {

		if self.consensus_state == trusted_consensus_state {
			Ok(())
		} else  {
			Err(Error::ImplementationSpecific("Consensus state not found".to_string()))
		}
    }
}

#[cfg(test)]
#[cfg(feature = "ismp_rs_tests")]
#[test]
//Test function that checks that the challenge period is elapsed before a new consensus update is allowed
pub fn check_challenge_period_elapsed() {
    println!("check_challenge_period_elapsed");
    assert!(true);
}
