use crate::{
    check_challenge_period, check_client_expiry,
    mocks::{Host, MockRouter},
    write_outgoing_commitments,
};
use std::sync::Arc;

#[test]
fn check_for_duplicate_requests_and_responses() {
    let host = Arc::new(Host::default());
    let router = MockRouter(host.clone());
    write_outgoing_commitments(&*host, &router).unwrap();
}

#[test]
fn should_reject_updates_within_challenge_period() {
    let host = Host::default();
    check_challenge_period(&host, [1u8; 4], vec![], vec![]).unwrap()
}

#[test]
fn should_reject_expired_check_clients() {
    let host = Host::default();
    check_client_expiry(&host, [1u8; 4], vec![], vec![]).unwrap()
}
