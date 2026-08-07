mod discovery;
mod crypto;

use std::thread;
use std::time::Duration;

use discovery::BeaconEther;
use discovery::packet::{NCNRPPacket, ResponsePayload};

fn main() {
    let own_discovery_packet = ResponsePayload::new(
        900, 
        8086, 
        String::from("darwin-aarch64-macosx"), 
        [0u8; 32],
    String::from("LHunter's-Node"), 
    );
    let ether = BeaconEther::new(own_discovery_packet);
    
}