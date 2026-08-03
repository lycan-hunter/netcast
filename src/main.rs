
mod discovery;
mod crypto;

use core::time;
use std::thread::sleep;

use discovery::BeaconEther;

use discovery::packet::{NCNRPPacket, ResponsePayload};

use crypto::Identity;
use rustls::client::Tls12Resumption::Disabled;




fn main() {
    let mut p = ResponsePayload::default();
    p.name = String::from("NEW-NODE");
    p.os_family = String::from("macosx");
    p.pub_key_hash = [0u8; 32];
    p.tcp_port = 6767;
let a = NCNRPPacket::Response(p.clone());
let b = NCNRPPacket::deserialize(&a.serialize()).unwrap();


let c = NCNRPPacket::Request;
let x = c.serialize();
let z = NCNRPPacket::deserialize(&x).unwrap();
println!("{:?}", c);
println!("{:?}", z);
}

