use crate::{error::Error, host::ChainID, prelude::Vec};
use codec::{Decode, Encode};

/// The ISMP POST request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct POST {
    /// The source state machine of this request.
    pub source_chain: ChainID,
    /// The destination state machine of this request.
    pub dest_chain: ChainID,
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Moudle Id of the sending module
    pub from: Vec<u8>,
    /// Module ID of the receiving module
    pub to: Vec<u8>,
    /// Timestamp which this request expires by.
    pub timeout_timestamp: u64,
    /// Encoded Request.
    pub data: Vec<u8>,
}

/// The ISMP GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct GET {
    /// The source state machine of this request.
    pub source_chain: ChainID,
    /// The destination state machine of this request.
    pub dest_chain: ChainID,
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Moudle Id of the sending module
    pub from: Vec<u8>,
    /// Storage keys that this request is interested in.
    pub keys: Vec<Vec<u8>>,
    /// Timestamp which this request expires by.
    pub timeout_timestamp: u64,
}

/// The ISMP request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum Request {
    /// A post request allows a module on a state machine to send arbitrary bytes to another module
    /// living in another state machine.
    Post(POST),
    /// A get request allows a module on a state machine to read the storage of another module
    /// living in another state machine.
    Get(GET),
}

impl Request {
    /// Get the source chain
    pub fn source_chain(&self) -> ChainID {
        match self {
            Request::Get(get) => get.source_chain,
            Request::Post(post) => post.source_chain,
        }
    }

    /// Get the destination chain
    pub fn dest_chain(&self) -> ChainID {
        match self {
            Request::Get(get) => get.dest_chain,
            Request::Post(post) => post.dest_chain,
        }
    }

    /// Get the request nonce
    pub fn nonce(&self) -> u64 {
        match self {
            Request::Get(get) => get.nonce,
            Request::Post(post) => post.nonce,
        }
    }

    /// Get the POST request data
    pub fn data(&self) -> Option<Vec<u8>> {
        match self {
            Request::Get(_) => None,
            Request::Post(post) => Some(post.data.clone()),
        }
    }

    /// Get the GET request keys.
    pub fn keys(&self) -> Option<Vec<Vec<u8>>> {
        match self {
            Request::Post(_) => None,
            Request::Get(get) => Some(get.keys.clone()),
        }
    }
}

/// The ISMP response
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Response {
    /// The reuest that triggered this response.
    pub request: Request,
    /// The response message.
    pub response: Vec<u8>,
}

/// This is the concrete type for Get requests
pub type GetResponse = Vec<(Vec<u8>, Vec<u8>)>;

/// Convenience enum for membership verification.
pub enum RequestResponse {
    Request(Request),
    Response(Response),
}

pub trait IISMPRouter {
    /// Dispatch a request from a module to the ISMP router.
    /// If request source chain is the host, it should be committed in state as a sha256 hash
    fn dispatch(&self, request: Request) -> Result<(), Error>;

    /// Provide a response to a previously received request.
    /// If response source chain is the host, it should be committed in state as a sha256 hash
    fn write_response(&self, response: Response) -> Result<(), Error>;
}
