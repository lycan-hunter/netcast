
mod discovery;
mod crypto;



use core::time;
use std::thread::sleep;

use discovery::{BeaconEther, BeaconPacket};
use crypto::Identity;



fn main() {

    let a = BeaconPacket::new();
    let b = BeaconEther::new(a).unwrap();
    let data = vec![9u8; 256];
    loop {
    b.send_beacon(&data);
    println!("Sent!");
    sleep(time::Duration::from_secs(2));
    }

}

