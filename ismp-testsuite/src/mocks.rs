use ismp::{
    consensus::{
        ConsensusClient, ConsensusClientId, IntermediateState, StateCommitment, StateMachineHeight,
        StateMachineId,
    },
    error::Error,
    host::{ISMPHost, StateMachine},
    messaging::Proof,
    router::{DispatchResult, DispatchSuccess, ISMPRouter, Request, RequestResponse, Response},
    util::{hash_request, hash_response},
};
use primitive_types::H256;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct MockClient;

#[derive(codec::Encode, codec::Decode)]
pub struct MockConsensusState {
    frozen_height: Option<u64>,
}

impl ConsensusClient for MockClient {
    fn verify_consensus(
        &self,
        _host: &dyn ISMPHost,
        _trusted_consensus_state: Vec<u8>,
        _proof: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<IntermediateState>), Error> {
        Ok(Default::default())
    }

    fn unbonding_period(&self) -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn verify_membership(
        &self,
        _host: &dyn ISMPHost,
        _item: RequestResponse,
        _root: StateCommitment,
        _proof: &Proof,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn state_trie_key(&self, _request: RequestResponse) -> Vec<Vec<u8>> {
        Default::default()
    }

    fn verify_state_proof(
        &self,
        _host: &dyn ISMPHost,
        _keys: Vec<Vec<u8>>,
        _root: StateCommitment,
        _proof: &Proof,
    ) -> Result<Vec<Option<Vec<u8>>>, Error> {
        Ok(Default::default())
    }

    fn is_frozen(&self, _trusted_consensus_state: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Host {
    requests: RefCell<BTreeSet<H256>>,
    responses: RefCell<BTreeSet<H256>>,
    consensus_states: RefCell<HashMap<ConsensusClientId, Vec<u8>>>,
    state_commitments: RefCell<HashMap<StateMachineHeight, StateCommitment>>,
    consensus_update_time: RefCell<HashMap<ConsensusClientId, Duration>>,
    frozen_state_machines: RefCell<HashMap<StateMachineId, StateMachineHeight>>,
    latest_state_height: RefCell<HashMap<StateMachineId, u64>>,
}

impl ISMPHost for Host {
    fn host_state_machine(&self) -> StateMachine {
        StateMachine::Polkadot(1000)
    }

    fn latest_commitment_height(&self, id: StateMachineId) -> Result<StateMachineHeight, Error> {
        self.latest_state_height
            .borrow()
            .get(&id)
            .map(|height| StateMachineHeight { id, height: *height })
            .ok_or_else(|| Error::ImplementationSpecific("latest height not found".into()))
    }

    fn state_machine_commitment(
        &self,
        height: StateMachineHeight,
    ) -> Result<StateCommitment, Error> {
        self.state_commitments
            .borrow()
            .get(&height)
            .map(|val| val.clone())
            .ok_or_else(|| Error::ImplementationSpecific("state commitment not found".into()))
    }

    fn consensus_update_time(&self, id: ConsensusClientId) -> Result<Duration, Error> {
        self.consensus_update_time
            .borrow()
            .get(&id)
            .map(|timestamp| *timestamp)
            .ok_or_else(|| Error::ImplementationSpecific("Consensus update time not found".into()))
    }

    fn consensus_state(&self, id: ConsensusClientId) -> Result<Vec<u8>, Error> {
        self.consensus_states
            .borrow()
            .get(&id)
            .map(|val| val.clone())
            .ok_or_else(|| Error::ImplementationSpecific("consensus state not found".into()))
    }

    fn timestamp(&self) -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    fn is_frozen(&self, height: StateMachineHeight) -> Result<bool, Error> {
        let val = self
            .frozen_state_machines
            .borrow()
            .get(&height.id)
            .map(|frozen_height| frozen_height >= &height)
            .unwrap_or(false);
        Ok(val)
    }

    fn request_commitment(&self, req: &Request) -> Result<H256, Error> {
        let hash = hash_request::<Self>(req);
        self.requests
            .borrow()
            .contains(&hash)
            .then(|| hash)
            .ok_or_else(|| Error::ImplementationSpecific("Request commitment not found".into()))
    }

    fn store_consensus_state(&self, id: ConsensusClientId, state: Vec<u8>) -> Result<(), Error> {
        self.consensus_states.borrow_mut().insert(id, state);
        Ok(())
    }

    fn store_consensus_update_time(
        &self,
        id: ConsensusClientId,
        timestamp: Duration,
    ) -> Result<(), Error> {
        self.consensus_update_time.borrow_mut().insert(id, timestamp);
        Ok(())
    }

    fn store_state_machine_commitment(
        &self,
        height: StateMachineHeight,
        state: StateCommitment,
    ) -> Result<(), Error> {
        self.state_commitments.borrow_mut().insert(height, state);
        Ok(())
    }

    fn freeze_state_machine(&self, height: StateMachineHeight) -> Result<(), Error> {
        self.frozen_state_machines.borrow_mut().insert(height.id, height);
        Ok(())
    }

    fn store_latest_commitment_height(&self, height: StateMachineHeight) -> Result<(), Error> {
        self.latest_state_height.borrow_mut().insert(height.id, height.height);
        Ok(())
    }

    fn delete_request_commitment(&self, req: &Request) -> Result<(), Error> {
        let hash = hash_request::<Self>(req);
        self.requests.borrow_mut().remove(&hash);
        Ok(())
    }

    fn consensus_client(&self, id: ConsensusClientId) -> Result<Box<dyn ConsensusClient>, Error> {
        Ok(Box::new(MockClient::default()))
    }

    fn keccak256(bytes: &[u8]) -> H256
    where
        Self: Sized,
    {
        sp_core::keccak_256(bytes).into()
    }

    fn challenge_period(&self, _id: ConsensusClientId) -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn ismp_router(&self) -> Box<dyn ISMPRouter> {
        todo!()
    }
}

#[derive(Default)]
pub struct MockRouter;

impl ISMPRouter for MockRouter {
    fn dispatch(&self, request: Request) -> DispatchResult {
        let host = Host::default();
        if request.dest_chain() != host.host_state_machine() {
            let hash = hash_request::<Host>(&request);
            host.requests.borrow_mut().insert(hash);
        }

        Ok(DispatchSuccess {
            dest_chain: request.dest_chain(),
            source_chain: request.source_chain(),
            nonce: request.nonce(),
        })
    }

    fn dispatch_timeout(&self, request: Request) -> DispatchResult {
        Ok(DispatchSuccess {
            dest_chain: request.dest_chain(),
            source_chain: request.source_chain(),
            nonce: request.nonce(),
        })
    }

    fn write_response(&self, response: Response) -> DispatchResult {
        let host = Host::default();
        if response.request.source_chain() != host.host_state_machine() {
            let hash = hash_response::<Host>(&response);
            host.responses.borrow_mut().insert(hash);
        }

        Ok(DispatchSuccess {
            dest_chain: response.request.dest_chain(),
            source_chain: response.request.source_chain(),
            nonce: response.request.nonce(),
        })
    }
}
