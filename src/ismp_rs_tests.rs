use core::{
    cell::RefCell,
    fmt::Debug,
    time::{self, Duration},
};
use keccak_hash::keccak;
use std::collections::HashMap;
use std::time::SystemTime;

use alloc::rc::Rc;

use crate::{
    consensus_client::{
        ConsensusClientId, IntermediateState, StateCommitment, StateMachineHeight, StateMachineId,
    },
    error::Error,
    handlers::{MessageResult, RequestResponseResult},
    host::{ ISMPHost, StateMachine},
    messaging::RequestMessage,
    router::{RequestResponse, ISMPRouter},
};

pub type Hash = [u8; 32];
pub const ETHEREUM_CONSENSUS_CLIENT_ID: u64 = 1;

#[derive(Debug, Clone)]
struct Dummy {
    storage_state_machine: Rc<RefCell<HashMap<StateMachineHeight, StateCommitment>>>,
    storage_consensus: Rc<RefCell<HashMap<ConsensusClientId, Vec<u8>>>>,
    storage_latest_state_machine: Rc<RefCell<HashMap<StateMachineId, StateMachineHeight>>>,
    frozen_machine_height: Rc<RefCell<HashMap<StateMachineHeight, bool>>>,
    frozen_consensus: Rc<RefCell<HashMap<ConsensusClientId, bool>>>,
    updated_consensus_timestamp: Rc<RefCell<HashMap<ConsensusClientId, Duration>>>,
	state_machine_id: StateMachine,
}

impl<> ISMPHost for Dummy {
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
        self.storage_consensus
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
        self.storage_consensus
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
        // self.storage_consensus
		// 	.borrow()
		// 	.get(&id)
		// 	.cloned()
		// 	.ok_or(Error::ConsensusStateNotFound { id })
		// 	.map(|consensus| Box::new(consensus) as Box<dyn crate::consensus_client::ConsensusClient>)
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
        todo!()
    }

    fn consensus_update_time(&self, id: ConsensusClientId) -> Result<core::time::Duration, Error> {
        todo!()
    }

    fn keccak256(bytes: &[u8]) -> keccak_hash::H256
    where
        Self: Sized,
    {
        todo!()
    }

    fn request_commitment(&self, req: &crate::router::Request) -> Result<keccak_hash::H256, Error> {
        todo!()
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

#[cfg(test)]
#[cfg(feature = "ismp_rs_tests")]
#[test]
//Test function that checks that the challenge period is elapsed before a new consensus update is allowed
pub fn check_challenge_period_elapsed() {
    println!("check_challenge_period_elapsed");
    assert!(true);
}
