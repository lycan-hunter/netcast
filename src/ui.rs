use log::warn;

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;

pub const NETCAST_UNIX_SOCKET_PATH: &str = "/tmp/netcast.sock";

pub struct DaemonEther {
    connects: i128,
    pub sock: UnixListener,
}

impl DaemonEther {
    pub async fn new() -> std::io::Result<Self> {
        if Path::new(NETCAST_UNIX_SOCKET_PATH).exists() {
            std::fs::remove_file(NETCAST_UNIX_SOCKET_PATH)?;
        }

        let sock = UnixListener::bind(NETCAST_UNIX_SOCKET_PATH)?;

        Ok(DaemonEther {
            connects: 0i128,
            sock,
        })
    }

    
}

enum DaemonCommands {
    GetNodesLists,
    

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