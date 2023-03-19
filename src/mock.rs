use std::collections::HashMap;
use crate::consensus_client::{StateMachineId, StateMachineHeight, ConsensusClientId, ConsensusClient, IntermediateState, ETHEREUM_CONSENSUS_CLIENT_ID};
use crate::{host, consensus_client::StateCommitment, error::Error, router::{Response, Request}, router::IISMPRouter};

use std::cell::RefCell;
use std::time::Duration;

pub struct HP {
    pub state: RefCell<HashMap<StateMachineId, StateMachineHeight>>,
    pub consensus: RefCell<HashMap<ConsensusClientId, StateCommitment>>,
    pub commits: RefCell<HashMap<StateMachineHeight, StateCommitment>>,
    pub req_commit: RefCell<HashMap<Request, StateCommitment>>,
    pub res_commit: RefCell<HashMap<Response, StateCommitment>>,
    pub timestamp: Duration,
    pub router: Stage,
    pub client: RefCell<HashMap<ConsensusClientId, Ethereum>>,
    pub mc: RefCell<HashMap<StateMachineId, ConsensusClientId>>
}

impl host::ISMPHost for HP {
    fn host(&self) -> host::ChainID {
        host::ChainID::HYPERSPACE
    }

    fn latest_commitment_height(&self, id: StateMachineId) -> Result<StateMachineHeight, Error> {
        todo!()
    }
    
    fn state_machine_commitment(&self, height: StateMachineHeight) -> Result<StateCommitment, Error> {
        todo!()
    }

    fn consensus_update_time(&self, id: ConsensusClientId) -> Result<Duration, Error> {
        todo!()
    }

    fn state_machine_update_time(&self, height: StateMachineHeight) -> Result<Duration, Error> {
        todo!()
    }

    fn consensus_state(&self, id: ConsensusClientId) -> Result<Vec<u8>, Error> {
        match self.consensus
        .borrow()
        .get(&id)
        .ok_or(Error::CannotHandleConsensusMessage).cloned() {
            Ok(val) => Ok(val.commitment_root),
            Err(e) => Err(e)
        }
    }

    fn host_timestamp(&self) -> Duration {
        self.timestamp
    }

    fn is_frozen(&self, height: StateMachineHeight) -> Result<bool, Error> {
        todo!()
    }

    fn request_commitment(&self, req: &Request) -> Result<Vec<u8>, Error> {
        match self.req_commit
            .borrow()
            .get(&req)
            .ok_or(Error::CannotHandleConsensusMessage)
            .cloned() 
        {
            Ok(val) => Ok(val.commitment_root),
            Err(e) => Err(e)
        }
    }

    fn response_commitment(&self, res: &Response) -> Result<Vec<u8>, Error> {
        match self.res_commit
            .borrow()
            .get(&res)
            .ok_or(Error::CannotHandleConsensusMessage)
            .cloned()
        {
            Ok(val) => Ok(val.commitment_root),
            Err(e) => Err(e)
        }
    }

    fn store_consensus_state(&mut self, id: ConsensusClientId, state: Vec<u8>) -> Result<(), Error> {
       match self.consensus.get_mut().get_mut(&id) {
        Some(val) => {
            val.commitment_root = state;
            Ok(())
        },
        None => Err(Error::CannotHandleConsensusMessage)
       }
    }

    fn store_consensus_update_time(
        &mut self,
        id: ConsensusClientId,
        timestamp: Duration,
    ) -> Result<(), Error> {
        match self.consensus.get_mut().get_mut(&id) {
            Some(val) => {
                val.timestamp = timestamp.as_secs();
                Ok(())
            },
            None => Err(Error::CannotHandleConsensusMessage)
        }
    }

    fn store_state_machine_update_time(
        &mut self,
        height: StateMachineHeight,
        timestamp: Duration,
    ) -> Result<(), Error> {
        match self.commits.get_mut().get_mut(&height) {
            Some(val) => {
                val.timestamp = timestamp.as_secs();
                Ok(())
            },
            None => Err(Error::CannotHandleConsensusMessage)
        }
    }

    fn store_state_machine_commitment(
        &mut self,
        height: StateMachineHeight,
        state: StateCommitment,
    ) -> Result<(), Error> {
        match self.commits.get_mut().get_mut(&height) {
            Some(val) => {
                val.timestamp = state.timestamp;
                val.commitment_root = state.commitment_root;
                Ok(())
            },
            None => Err(Error::CannotHandleConsensusMessage)
        }
    }

    fn freeze_state_machine(&mut self, height: StateMachineHeight) -> Result<(), Error> {
        todo!()
    }

    fn consensus_client(&self, id: ConsensusClientId) -> Result<Box<dyn ConsensusClient>, Error> {
        match self.client.borrow().get(&id).ok_or(Error::CannotHandleConsensusMessage).cloned() {
            Ok(val) => Ok(Box::new(val)),
            Err(e) => Err(e)
        }
    }

    fn keccak256(&self, bytes: &[u8]) -> [u8; 32] {
        todo!()
    }

    fn delay_period(&self, id: StateMachineId) -> Duration {
        todo!()
    }

    fn client_id_from_state_id(&self, id: StateMachineId) -> Result<ConsensusClientId, Error> {
        self.mc.borrow().get(&id).ok_or(Error::CannotHandleConsensusMessage).cloned()
    }

    fn ismp_router(&self) -> Box<dyn IISMPRouter> {
        Box::new(self.router.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Ethereum;
impl ConsensusClient for Ethereum {
    fn verify(
        &self,
        host: &dyn host::ISMPHost,
        trusted_consensus_state: Vec<u8>,
        proof: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<IntermediateState>), Error> {
        let vec: Vec<u8> = Vec::new();
        let intermediate: Vec<IntermediateState> = Vec::new();
        Ok((vec, intermediate))
    }

    fn consensus_id(&self) -> ConsensusClientId {
        ETHEREUM_CONSENSUS_CLIENT_ID
    }

    fn unbonding_period(&self) -> Duration {
        todo!()
    }

    fn verify_membership(
        &self,
        host: &dyn host::ISMPHost,
        key: Vec<u8>,
        commitment: Vec<u8>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_non_membership(
        &self,
        host: &dyn host::ISMPHost,
        key: Vec<u8>,
        commitment: Vec<u8>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn is_frozen(&self, host: &dyn host::ISMPHost, id: ConsensusClientId) -> Result<bool, Error> {
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
