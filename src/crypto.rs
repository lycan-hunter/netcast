use zeroize::Zeroize;
use std::fs;
use std::io::Read;
use log::{error, info, debug};

use crate::discovery::packet::NCNRPPacket;


const NETCASTKEY_PATH: &str = "~/.netcastrc";
struct PeerHello {
    pub_key: [u8; 32],

}

#[derive(Zeroize)]
struct OwnSecret {
    bytes: [u8; 32],
}



impl OwnSecret {
    pub async fn new() -> std::io::Result<Self> { 
        match fs::exists(NETCASTKEY_PATH) {
            Ok(is_exists) => {
                // Creating file if not exists
                if !is_exists {
                    match fs::File::create(NETCASTKEY_PATH) {
                        Ok(_) => debug!("Created '{}'", NETCASTKEY_PATH),
                        Err(msg) => error!("Failed to create '{}': {}", NETCASTKEY_PATH, msg),
                    }
                }
                


            },
            Err(msg) => error!("Failed to search for '{}': {}", NETCASTKEY_PATH, msg),
        }
        Ok(Self{
            bytes: [0u8; 32],
        })
    }
}