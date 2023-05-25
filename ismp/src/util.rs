//! ISMP utilities

use crate::{
    host::IsmpHost,
    router::{Request, Response},
};
use alloc::{string::ToString, vec::Vec};
use codec::Encode;
use primitive_types::H256;

/// Return the keccak256 hash of a request
pub fn hash_request<H: IsmpHost>(req: &Request) -> H256 {
    match req {
        Request::Post(post) => {
            let mut buf = Vec::new();

            let source_chain = post.source_chain.to_string();
            let dest_chain = post.dest_chain.to_string();
            let nonce = post.nonce.to_be_bytes();
            let timestamp = post.timeout_timestamp.to_be_bytes();
            buf.extend_from_slice(source_chain.as_bytes());
            buf.extend_from_slice(dest_chain.as_bytes());
            buf.extend_from_slice(&nonce);
            buf.extend_from_slice(&timestamp);
            buf.extend_from_slice(&post.from);
            buf.extend_from_slice(&post.to);
            buf.extend_from_slice(&post.data);
            H::keccak256(&buf[..])
        }
        Request::Get(get) => {
            let mut buf = Vec::new();

            let source_chain = get.source_chain.to_string();
            let dest_chain = get.dest_chain.to_string();
            let nonce = get.nonce.to_be_bytes();
            let height = get.height.to_be_bytes();
            let timestamp = get.timeout_timestamp.to_be_bytes();
            buf.extend_from_slice(source_chain.as_bytes());
            buf.extend_from_slice(dest_chain.as_bytes());
            buf.extend_from_slice(&nonce);
            buf.extend_from_slice(&height);
            buf.extend_from_slice(&timestamp);
            buf.extend_from_slice(&get.from);
            buf.extend_from_slice(&get.keys.encode());
            H::keccak256(&buf[..])
        }
    }
}

/// Return the keccak256 of a response
pub fn hash_response<H: IsmpHost>(res: &Response) -> H256 {
    let (req, response) = match res {
        Response::Post(res) => (&res.post, &res.response),
        // Responses to get messages are never hashed
        _ => return Default::default(),
    };
    let mut buf = Vec::new();
    let source_chain = req.source_chain.to_string();
    let dest_chain = req.dest_chain.to_string();
    let nonce = req.nonce.to_be_bytes();
    let timestamp = req.timeout_timestamp.to_be_bytes();
    buf.extend_from_slice(source_chain.as_bytes());
    buf.extend_from_slice(dest_chain.as_bytes());
    buf.extend_from_slice(&nonce);
    buf.extend_from_slice(&timestamp);
    buf.extend_from_slice(&req.data);
    buf.extend_from_slice(&req.from);
    buf.extend_from_slice(&req.to);
    buf.extend_from_slice(response);
    H::keccak256(&buf[..])
}
