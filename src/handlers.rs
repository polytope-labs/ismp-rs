use crate::{
    consensus_client::{ConsensusClient, ConsensusClientId, StateMachineHeight},
    error::Error,
    host::{ChainID, ISMPHost},
    messaging::{Message, Proof},
};
use alloc::{boxed::Box, collections::BTreeSet};

mod consensus;
mod request;
mod response;

pub struct ConsensusUpdateResult {
    /// Consensus client Id
    pub consensus_client_id: ConsensusClientId,
    /// Tuple of previous latest height and new latest height for a state machine
    pub state_updates: BTreeSet<(StateMachineHeight, StateMachineHeight)>,
}

pub struct RequestResponseResult {
    /// Destination chain for request or response
    pub dest_chain: ChainID,
    /// Source chain for request or response
    pub source_chain: ChainID,
    /// Request nonce
    pub nonce: u64,
}

/// Result returned when ismp messages are handled successfully
pub enum MessageResult {
    ConsensusMessage(ConsensusUpdateResult),
    Request(RequestResponseResult),
    Response(RequestResponseResult),
}

/// This function serves as an entry point to handle the message types provided by the ISMP protocol
/// Does not handle create consensus client message.
pub fn handle_incoming_message(
    host: &dyn ISMPHost,
    message: Message,
) -> Result<MessageResult, Error> {
    match message {
        Message::Consensus(consensus_message) => consensus::handle(host, consensus_message),
        Message::Request(req) => request::handle(host, req),
        Message::Response(resp) => response::handle(host, resp),
        _ => Err(Error::CannotHandleConsensusMessage),
    }
}

/// This function checks to see that the delay period configured on the host chain
/// for the state machine has elasped.
fn verify_delay_passed(
    host: &dyn ISMPHost,
    proof_height: StateMachineHeight,
) -> Result<bool, Error> {
    let update_time = host.consensus_update_time(proof_height.id.consensus_client)?;
    let delay_period = host.delay_period(proof_height.id.consensus_client);
    let current_timestamp = host.host_timestamp();
    Ok(current_timestamp - update_time > delay_period)
}

/// This function does the preliminary checks for a request or response message
/// - It ensures the consensus client is not frozen
/// - It ensures the state machine is not frozen
/// - Checks that the delay period configured for the state machine has elaspsed.
fn validate_state_machine(
    host: &dyn ISMPHost,
    proof: &Proof,
) -> Result<Box<dyn ConsensusClient>, Error> {
    // Ensure consensus client is not frozen
    let consensus_client_id = proof.height.id.consensus_client;
    let consensus_client = host.consensus_client(consensus_client_id)?;
    if consensus_client.is_frozen(host, consensus_client_id)? {
        return Err(Error::FrozenConsensusClient { id: consensus_client_id })
    }

    // Ensure state machine is not frozen
    if host.is_frozen(proof.height)? {
        return Err(Error::FrozenStateMachine { height: proof.height })
    }

    // Ensure delay period has elapsed
    if !verify_delay_passed(host, proof.height)? {
        return Err(Error::DelayNotElapsed {
            current_time: host.host_timestamp(),
            update_time: host.consensus_update_time(proof.height.id.consensus_client)?,
        })
    }

    Ok(consensus_client)
}
