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
use discovery::packet::{NCNRPPacket, ResponsePayload, HeartbeatPayload};


#[tokio::main]
async fn main() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None) 
        .format_module_path(false) 
        .write_style(WriteStyle::Auto) 
        .init();

    // let mut OWN_DEADLINE = time::Instant();

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

            Ok((NCNRPPacket::Response(_), addr)) => {
                info!("Recieved response packet from '{}'", addr.to_string());
            }

            Ok((NCNRPPacket::Heartbreat(hp), addr)) => {
                if !ether_rr.is_in_found_nodes(&hp.pub_key_hash) {
                    match ether_rr.send_request_to(addr).await {
                        Err(msg) => error!("Failed to send request to '{}': {}", addr.to_string(), msg),
                        _ => (),
                    }
                }
            }
            
            Err(msg) => error!("Failed to recieve packet: {}", msg),
        }
    }
});

    // Task 2: garbadge cleaner
        let ether_gc = ether.clone();
        tokio::spawn( async move {
            let mut tick = tokio::time::interval(time::Duration::from_secs(3u64));
            loop {
                tick.tick().await;
                ether_gc.found_nodes_gc();
            }
        });

}
