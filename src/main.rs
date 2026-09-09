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
    Builder::from_env(Env::default().default_filter_or("debug"))
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
    String::from("NetCast's-Node2"), 
    );

    let ether: Arc<BeaconEther>;
    loop {
        match BeaconEther::new_with_port(own_discovery_packet.clone(), 31337).await {
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

    // First broadcast announcement
    let result_ann = ether.send_response_broadcast().await;
    
    for iface in result_ann {
        match iface {
            Ok(_) => (),
            Err(msg) => 
                error!("Failed to announcing oneself on one of interfaces: {}", msg),
        }
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
                    info!("Received response packet from '{}'", addr.to_string());
                    debug!("{:?}", ether_rr.found_nodes());
                }

                Ok((NCNRPPacket::Heartbeat(hp), addr)) => {
                    debug!("Received heartbeat packet from '{}'", addr.to_string());
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

    // Task 2: garbage cleaner
        let ether_gc = ether.clone();
        tokio::spawn( async move {
            let mut tick = tokio::time::interval(time::Duration::from_secs(3u64));
            loop {
                tick.tick().await;
                ether_gc.found_nodes_gc();
                debug!("Found nodes pool was cleared by GC");
            }
        });

        let ether_hb = ether.clone();
        tokio::spawn(async move {
            let mut hb_tick = tokio::time::interval(time::Duration::from_secs(5));
            loop {
                hb_tick.tick().await;
                match ether_hb.send_heartbeat_broadcast().await {
                    results => {
                        for result in results {
                            if let Err(msg) = result {
                                error!("Failed to send heartbeat packet: {}", msg);
                            }
                        }
                    }
                }
                debug!("Sent heartbeat packet");
            }
        });

    
    info!("NetCast Deamon is up !");
    loop {
        tokio::time::sleep(time::Duration::from_secs(3600)).await;
    }
}
