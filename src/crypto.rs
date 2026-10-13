use ring::agreement::EphemeralPrivateKey;
use ring::agreement::PublicKey;
use ring::signature::Ed25519KeyPair;
// use zeroize::Zeroize;
use std::fs;
// use std::io::Read;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use log::error;

use ring::signature::KeyPair;
use ring::signature;

use ring::agreement;
use ring::rand::{SecureRandom, SystemRandom};

use ring::digest;

const NETCASTKEY_PATH: &str = "./netcast.key";

// pub async fn verify_sign(message: &[u8], sign_bytes: &[u8]) -> Result<bool, _> {

// }

pub enum PeerHelloType {
    Incoming,
    Outgoing,
}


pub struct PeerHello {
    pub_key: [u8; 32],
    pkt_type: PeerHelloType,
    private_eph: EphemeralPrivateKey,
    public_eph: PublicKey,
}

impl PeerHello {
    pub async fn new(pub_key: [u8;32], pkt_type: PeerHelloType) -> std::io::Result<Self> {
        let rng = ring::rand::SystemRandom::new();
        let private_eph = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, 
                        "Failed to create ephemeral private key"))?;

        let public_eph = private_eph.compute_public_key().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, 
                        "Failed to create ephemeral public key from private"))?;
        Ok(Self {
            pub_key,
            pkt_type,
            private_eph,
            public_eph,
        })
    }

    pub async fn serialize(&self, key_pair: &Ed25519KeyPair) -> std::io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        
        let pub_eph_key_bytes = self.public_eph.as_ref();
        buf.extend_from_slice(pub_eph_key_bytes);
        buf.extend_from_slice(key_pair.sign(pub_eph_key_bytes).as_ref());
        Ok(buf)
    }
}


// #[derive(Zeroize)]
pub struct OwnSecret {
    key_pair: Ed25519KeyPair,
    pub_key_hash: [u8; 32],
}

impl OwnSecret {
    pub async fn new() -> std::io::Result<Self> { 
        match fs::exists(NETCASTKEY_PATH) {
            Ok(is_exists) => {
                // Creating file if not exists
                if !is_exists {
                    let path = Path::new(NETCASTKEY_PATH);
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let mut private_key = [0u8; 32];
                    let rng = SystemRandom::new();
                    rng.fill(&mut private_key).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, 
                        "Failed to fill seed with random bytes"))?;
                    
                    let mut key_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(NETCASTKEY_PATH)?;

                    key_file.write_all(&private_key)?;
                }
                
                let private_key_bytes = fs::read(NETCASTKEY_PATH)?;
                let key_pair = 
                    signature::Ed25519KeyPair::from_seed_unchecked(&private_key_bytes)
                        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, 
                        "Failed to create key pair from private key"))?;
                
                let pub_key_hash = digest::digest(&digest::SHA256, key_pair
                    .public_key().as_ref())
                    .as_ref().try_into().unwrap();
                
                return Ok(Self { key_pair, pub_key_hash});

            },
            Err(msg) => {
                error!("Failed to search for '{}': {}", NETCASTKEY_PATH, msg);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to search for '{}': {}", NETCASTKEY_PATH, msg)));

            },
        }
        
    }

    pub async fn get_key_pair(&self) -> &Ed25519KeyPair {
        &self.key_pair
    }
    pub async fn get_pub_key_hash(&self) -> &[u8;32] {
        &self.pub_key_hash
    }
}