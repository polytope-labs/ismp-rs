use codec::{Decode, Encode};
use std::collections::HashMap;

use crate::consensus_client::{
    ConsensusClient, ConsensusClientId, IntermediateState, StateMachineHeight, StateMachineId,
};
use crate::router::RequestResponse;
use crate::{
    consensus_client::StateCommitment,
    error::Error,
    host,
    router::ISMPRouter,
    router::{Request, Response},
};

use std::cell::RefCell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const POLKADOT_CONSENSUS_CLIENT_ID: ConsensusClientId = 300;

#[derive(Debug)]
pub struct Host {
    pub commits: RefCell<HashMap<StateMachineHeight, StateCommitment>>,
    pub req_commit: RefCell<HashMap<Request, StateCommitment>>,
    pub router: Stage,
    pub frozen: RefCell<Vec<StateMachineHeight>>,
    pub client: RefCell<HashMap<ConsensusClientId, Polkadot>>,
    pub latest_height: RefCell<HashMap<StateMachineId, StateMachineHeight>>,
}

impl host::ISMPHost for Host {
    fn host(&self) -> host::ChainID {
        host::ChainID::HYPERSPACE
    }

    fn latest_commitment_height(&self, id: StateMachineId) -> Result<StateMachineHeight, Error> {
        if let Some(height) = self.latest_height.borrow().get(&id) {
            return Ok(*height);
        } else {
            Ok(StateMachineHeight {
                id: StateMachineId { state_id: 0, consensus_client: 0 },
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

    fn consensus_update_time(&self, id: ConsensusClientId) -> Result<Duration, Error> {
        if let Some(client) = self.client.borrow().get(&id) {
            return Ok(Duration::from_secs(client.timestamp));
        } else {
            Err(Error::ImplementationSpecific("Consensus client is non existent".to_string()))
        }
    }

    fn consensus_state(&self, id: ConsensusClientId) -> Result<Vec<u8>, Error> {
        if let Some(state) = self.client.borrow().get(&id) {
            return Ok(state.state.clone());
        } else {
            Err(Error::ConsensusStateNotFound { id })
        }
    }

    fn timestamp(&self) -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
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
            return Ok(commit.ismp_root.to_vec());
        } else {
            return Err(Error::RequestCommitmentNotFound {
                nonce: req.nonce(),
                source: req.source_chain(),
                dest: req.dest_chain(),
            });
        }
    }

    fn store_consensus_state(&self, id: ConsensusClientId, state: Vec<u8>) -> Result<(), Error> {
        self.client.borrow_mut().get_mut(&id).map(|old_state| old_state.state = state).unwrap();
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
            .ok_or(Error::ImplementationSpecific("Client does not exist".to_string()))
            .cloned()
        {
            Ok(val) => Ok(Box::new(val)),
            Err(e) => Err(e),
        }
    }

    fn challenge_period(&self, id: ConsensusClientId) -> Duration {
        if let Some(client) = self.client.borrow().get(&id) {
            let (_, _, challenge_period): (Vec<IntermediateState>, bool, Duration) =
                Decode::decode(&mut client.state.as_ref()).unwrap();
            challenge_period
        } else {
            Duration::from_secs(0)
        }
    }

    fn keccak256(&self, _bytes: &[u8]) -> [u8; 32] {
        todo!()
    }

    fn ismp_router(&self) -> Box<dyn ISMPRouter> {
        Box::new(self.router.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Polkadot {
    pub(crate) id: ConsensusClientId,
    pub(crate) state: Vec<u8>, // includes the frozen state as a boolean and challenge period as Duration
    pub(crate) timestamp: u64,
}
impl ConsensusClient for Polkadot {
    fn verify_consensus(
        &self,
        host: &dyn host::ISMPHost,
        trusted_consensus_state: Vec<u8>,
        proof: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<IntermediateState>), Error> {
        let (states, _, _): (Vec<IntermediateState>, bool, Duration) =
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

    fn unbonding_period(&self) -> Duration {
        Duration::from_secs(7 * 24 * 60 * 60)
    }

    fn verify_membership(
        &self,
        _host: &dyn host::ISMPHost,
        _key: RequestResponse,
        _commitment: StateCommitment,
        _proof: &crate::messaging::Proof,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_state_proof(
        &self,
        _host: &dyn host::ISMPHost,
        _key: Vec<u8>,
        _root: StateCommitment,
        _proof: &crate::messaging::Proof,
    ) -> Result<Vec<u8>, Error> {
        todo!()
    }

    fn verify_non_membership(
        &self,
        _host: &dyn host::ISMPHost,
        _key: RequestResponse,
        _commitment: StateCommitment,
        _proof: &crate::messaging::Proof,
    ) -> Result<(), Error> {
        todo!()
    }

    fn is_frozen(&self, trusted_consensus_state: &[u8]) -> Result<(), Error> {
        let (_, is_frozen, _): (Vec<IntermediateState>, bool, Duration) =
            Decode::decode(&mut trusted_consensus_state.as_ref()).unwrap();

        if is_frozen {
            return Err(Error::FrozenConsensusClient { id: self.id });
        } else {
            return Ok(());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stage;
impl ISMPRouter for Stage {
    fn dispatch(&self, _request: Request) -> Result<(), Error> {
        todo!()
    }

    fn write_response(&self, _response: Response) -> Result<(), Error> {
        todo!()
    }
}
