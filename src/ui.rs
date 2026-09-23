use crate::discovery::packet::ResponsePayload;
use crate::discovery::BeaconEther;

use std::os::fd::{OwnedFd, AsRawFd}; 
use unix_ancillary::AsyncUnixStreamExt;
use log::{info, error, debug};

use tokio::net::{UnixListener, UnixStream};
use tokio::io::AsyncReadExt; 
use std::path::Path;
use std::sync::Arc;

use serde::{Serialize, Deserialize};

pub const NETCAST_UNIX_SOCKET_PATH: &str = "/tmp/netcast.sock";

pub struct DaemonEther {
    connects: i128,
    pub sock: UnixListener,
    network_ether: Arc<BeaconEther>,
}

impl DaemonEther {
    pub async fn new(network_ether: Arc<BeaconEther>) -> std::io::Result<Self> {
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

    pub async fn run_loop(self: Arc<Self>) {
        loop {
            match self.sock.accept().await {
                Ok((stream, _)) => {
                    info!("Incoming connection on netcast unix socket");
                    let this = self.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = this.handle_client(stream).await {
                            error!("Error handling unix client: {}", e);
                        }
                    });
                },
                Err(msg) => {
                    error!("Failed to accept connection: {}", msg);
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                },
            }
        }
    }

    pub async fn handle_client(&self, stream: UnixStream) -> std::io::Result<()> {
        let mut client_stream = stream;

        loop {
            let msg = match Self::read_client_message(&mut client_stream)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) 
            {
                Ok(msg) => msg,
                Err(e) => {
                    debug!("Client disconnected or read error: {}", e);
                    break; 
                }
            };

            match msg {
                ClientMessage::Connection => {
                    info!("Client initialized handshake connection.");
                }

                ClientMessage::GetFoundNodes => {
                    let nodes = self.network_ether.found_nodes(); 
                    let response = DaemonMessage::FoundNodes(nodes);
                    
                    Self::send_daemon_message(&mut client_stream, &response)
                        .await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                }

                ClientMessage::CreateConnection(node_pubkey_hash) => {
                    info!("Client requested connection to node hash: {:?}", node_pubkey_hash);
                }

                ClientMessage::TrustConfirm(is_trusted) => {
                    info!("Client sent trust confirmation response: {}", is_trusted);
                }

                ClientMessage::CloseConnection => {
                    debug!("Connection closed by unisock message");
                    break; 
                }
            }
        }
        
        Ok(())
    }

    async fn read_client_message(stream: &mut UnixStream) -> Result<ClientMessage, Box<dyn std::error::Error + Send + Sync>> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        let msg: ClientMessage = bincode::deserialize(&buf)?;
        Ok(msg)
    }

    async fn send_daemon_message(stream: &mut UnixStream, msg: &DaemonMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = bincode::serialize(msg)?;
        let len = payload.len() as u32;
        
        let mut full_packet = len.to_le_bytes().to_vec();
        full_packet.extend_from_slice(&payload);

        if let DaemonMessage::SessionData { fd: Some(owned_fd), .. } = msg {
            stream.send_fds_all(&full_packet, &[owned_fd]).await?;
        } else {
            use tokio::io::AsyncWriteExt;
            stream.write_all(&full_packet).await?;
            stream.flush().await?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum DaemonMessage {
    FoundNodes(Vec<ResponsePayload>),
    ConnectionResult(bool),
    TrustConfirm(bool),
    SessionData { 
        key: Vec<u8>, 
        #[serde(skip)] 
        fd: Option<OwnedFd> 
    },
}

#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    Connection,
    GetFoundNodes,
    CreateConnection([u8; 32]),
    TrustConfirm(bool),
    CloseConnection,
}
