mod discovery;
mod crypto;
mod channel;

use log::{error, warn, info, debug, trace};
use rand::Rng;
use env_logger::{Builder, Env, WriteStyle};
use std::thread;
use std::time;

use discovery::BeaconEther;
use discovery::packet::{NCNRPPacket, ResponsePayload};

fn main() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None) 
        .format_module_path(false) 
        .write_style(WriteStyle::Auto) 
        .init();

    let own_discovery_packet = ResponsePayload::new(
        900, 
        8086, 
        String::from("darwin-aarch64-macosx"), 
        [0u8; 32],
    String::from("NetCast's-Node"), 
    );

    let mut ether: BeaconEther;
    loop {
        match BeaconEther::new(own_discovery_packet.clone()) {
            Err(msg) => {
                error!("Failed to start ether object: {}", msg);
                info!("Retrying to start ether object...");
            },
            Ok(be) => {
                ether = be;
                break;
            }
        }
        thread::sleep(time::Duration::from_secs(3));
    }
    
    let mut rng = rand::rng();
    loop {
        match ether.recv_packet() {

            Ok((NCNRPPacket::Request, addr)) => {

                thread::sleep(time::Duration::from_millis(rng.random_range(16..=120)));

                match ether.send_response_to(addr) {
                    Err(msg) => error!("Failed to send response: {}", msg),
                    Ok(_) => (),
                }
            }

            Ok((NCNRPPacket::Response(_), addr)) => 
                info!("Recieved response packet from '{}'", addr.to_string()),
            
            Err(msg) => error!("Failed to recieve packet: {}", msg),
        }
    }

}