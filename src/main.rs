mod discovery;
// mod crypto;
mod channel;

use log::{error, warn, info, debug, trace};
use rand;
use env_logger::{Builder, Env, WriteStyle};
use std::thread;
use std::time;
use std::sync::Arc;

use discovery::BeaconEther;
use discovery::packet::{NCNRPPacket, ResponsePayload};

#[tokio::main]
async fn main() {
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

    let ether: Arc<BeaconEther>;
    loop {
        match BeaconEther::new(own_discovery_packet.clone()).await {
            Err(msg) => {
                error!("Failed to start netcast interface: {}", msg);
                info!("Retrying to start netcast interface...");
            },
            Ok(be) => {
                ether = Arc::new(be);
                break;
            }
        }
        thread::sleep(time::Duration::from_secs(3));
    }

    // Task 1: recieving/responsing
    let ether_rr = ether.clone();
    tokio::spawn( async move {
        loop {
        match ether_rr.recv_packet().await {

            Ok((NCNRPPacket::Request, addr)) => {

                tokio::time::sleep(time::Duration::from_millis(rand::random_range(16..=120))).await;

                match ether_rr.send_response_to(addr).await {
                    Err(msg) => error!("Failed to send response: {}", msg),
                    Ok(_) => (),
                }
            }

            Ok((NCNRPPacket::Response(rp), addr)) => {
                info!("Recieved response packet from '{}'", addr.to_string());
                match ether_rr.send_response_to(addr).await {
                    Err(msg) => error!("Failed to send response: {}", msg),
                    Ok(_) => (),
                }               
            }
            
            Err(msg) => error!("Failed to recieve packet: {}", msg),
        }
    }
});

}
