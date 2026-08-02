
mod discovery;
mod crypto;

use core::time;
use std::thread::sleep;

use discovery::{BeaconEther, BeaconPacket};

use discovery::packet::{NCNRPPacket, ResponsePayload};

use crypto::Identity;
use rustls::client::Tls12Resumption::Disabled;




fn main() {

    let a = BeaconPacket::new();
    let b = BeaconEther::new(a).unwrap();

    let data = NCNRPPacket::Request;

    // let data = vec![9u8; 256];
    loop {
    b.send_beacon_broadcast(&data.serialize());
    println!("Sent!");
    sleep(time::Duration::from_secs(2));

    let mut p = ResponsePayload::new();

    p.name = "NC-User-228)))".to_string();
    p.node_id = "ID-ID-ID-ID-ID-ID".to_string();
    p.os_family = "macosx-darwin-arm64                                                                                                                                                                                                                                                                           ".to_string();
    p.pub_key_hash = [0u8; 32];
    p.tcp_port = 6767;

    let data = NCNRPPacket::Response(p);
    b.send_beacon_broadcast(&data.serialize());
    println!("Sent!");
    sleep(time::Duration::from_secs(2));
    }

}

