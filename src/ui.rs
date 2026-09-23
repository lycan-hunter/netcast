use crate::discovery::packet::ResponsePayload;
use crate::ui::DaemonMessage::FoundNodes;
use crate::discovery::BeaconEther;

use log::{info, error};

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::os::fd::OwnedFd;
use std::path::Path;

pub const NETCAST_UNIX_SOCKET_PATH: &str = "/tmp/netcast.sock";

pub struct DaemonEther {
    connects: i128,
    pub sock: UnixListener,
    network_ether: BeaconEther,
}

impl DaemonEther {
    pub async fn new(network_ether: BeaconEther) -> std::io::Result<Self> {
        if Path::new(NETCAST_UNIX_SOCKET_PATH).exists() {
            std::fs::remove_file(NETCAST_UNIX_SOCKET_PATH)?;
        }

        let sock = UnixListener::bind(NETCAST_UNIX_SOCKET_PATH)?;

        Ok(Self {
            connects: 0i128,
            sock,
            network_ether,

        })
    }

    pub async fn run_loop(self: std::sync::Arc<Self>) {
        loop {
            let mut stream;
            match self.sock.accept().await {
                Ok((unix_stream, _)) => {
                    // stream = unix_stream;
                    info!("Incoming connection unisock connection");
                    let this = self.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = this.handle_client(stream).await {
                            error!("Error handling unix client: {}", e);
                        }
                    });
                },
                Err(msg) => {
                    error!("Failed to create connection: {}", msg);
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    continue;
                },
            };

            tokio::spawn( async move {
                let mut buf = [0u8; 1024];

                loop {
                    match stream.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {

                        },
                        Err(msg) => {
                            error!("Failed to listen unix socket's client: {}", msg);

                        }
                    }
                }
            });

        }
    }

    pub async fn send_message(msg_type: DaemonMessage) {
        match msg_type {
            FoundNodes(nodes) => {
                
            }
            
        }
    }

    
}

enum DaemonMessage {
    FoundNodes(Vec<ResponsePayload>),
    ConnectionResult(bool),
    TrustConfirm(bool),
    SessionData{ key: Vec<u8>, fd: OwnedFd},

}

// Asking found nodes
// Creating connections

// 1. Client -> Daemon: Connection

// 2. Client -> Daemon: Get found nodes
// 3. Client <- Daemon: Send found nodes data

// 4. Client -> Daemon: Create connection to X node
// 5. Client <- Daemon: Send connection result

// 6. Client <- Daemon: Ask to trust node's key
// 7. Client -> Daemon: Send trust confirmation

// 8. Client <- Daemon: Send session key + file descriptor of socket / Result