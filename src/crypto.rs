use zeroize::Zeroize;
use std::fs;
use std::io::Read;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

use log::{error, info, debug};

use ring::signature::KeyPair;
use ring::signature;

use ring::agreement;
use ring::rand::{SecureRandom, SystemRandom};

const NETCASTKEY_PATH: &str = "/etc/netcast/netcast.key";

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
                    match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o400)
                    .open(NETCASTKEY_PATH) {
                        Ok(_) => debug!("Created '{}'", NETCASTKEY_PATH),
                        Err(msg) => error!("Failed to create '{}': {}", NETCASTKEY_PATH, msg),
                    }
                    let mut private_key = [0u8; 32];
                    let rng = SystemRandom::new();
                    rng.fill(&mut private_key).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, 
                        "Failed to fill seed with random bytes"))?;
                    let mut key_file = fs::File::open(NETCASTKEY_PATH)?;
                    key_file.write_all(&private_key)?;

                }
                


            },
            Err(msg) => error!("Failed to search for '{}': {}", NETCASTKEY_PATH, msg),
        }
        Ok(Self{
            bytes: [0u8; 32],
        })
    }
}