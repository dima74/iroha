#![allow(clippy::pedantic, clippy::restriction)]

use std::thread;

use iroha_client::client::Client;
use iroha_core::config::Configuration;
use iroha_data_model::{peer::Peer as DataModelPeer, prelude::*};
use test_network::*;

#[test]
fn connected_peers_with_f_2_1_2() {
    connected_peers_with_f(2)
}

#[test]
// TODO This case does not have to be supported, but at least have to be error-handled
#[ignore]
fn connected_peers_with_f_1_0_1() {
    connected_peers_with_f(1)
}

/// Test the number of connected peers, changing the number of faults tolerated down and up
fn connected_peers_with_f(faults: u64) {
    let n_peers = 3 * faults + 1;
    let mut status;

    let (_rt, network, mut genesis_client) = <Network>::start_test_with_runtime(n_peers as u32, 1);
    wait_for_genesis_committed(network.clients(), 0);
    let pipeline_time = Configuration::pipeline_time();

    // Confirm all peers connected
    status = genesis_client.get_status().unwrap();
    assert_eq!(status.peers, n_peers - 1);
    assert_eq!(status.blocks, 1);

    // Unregister a peer: committed with f = `faults`
    // then `status.peers` decrements
    let peer = network.peers.values().last().unwrap();
    let peer_client = Client::test(&peer.api_address, &peer.status_address);
    let unregister_peer = UnregisterBox::new(IdBox::PeerId(peer.id.clone()));
    genesis_client.submit(unregister_peer).unwrap();
    thread::sleep(pipeline_time * 2);
    status = genesis_client.get_status().unwrap();
    assert_eq!(status.peers, n_peers - 2);
    assert_eq!(status.blocks, 2);
    status = peer_client.get_status().unwrap();
    assert_eq!(status.peers, 0);
    assert_eq!(status.blocks, 2);

    // Re-register the peer: committed with f = `faults` - 1
    // then `status.peers` increments
    let register_peer = RegisterBox::new(IdentifiableBox::Peer(
        DataModelPeer::new(peer.id.clone()).into(),
    ));
    genesis_client.submit(register_peer).unwrap();
    thread::sleep(pipeline_time * 2);
    status = genesis_client.get_status().unwrap();
    assert_eq!(status.peers, n_peers - 1);
    assert_eq!(status.blocks, 3);
    status = peer_client.get_status().unwrap();
    assert_eq!(status.peers, n_peers - 1);
    assert_eq!(status.blocks, 3);
}