mod discovery;
// mod crypto;
mod channel;

use log::{error, warn, info, debug, trace};
use rand;
use env_logger::{Builder, Env, WriteStyle};
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
    String::from("NetCast's-Node"), 
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
        tokio::time::sleep(time::Duration::from_secs(3)).await;
    }

    // First broadcast announcement
    let ether_ann = ether.clone();
    tokio::spawn(async move {
        let result_ann = ether_ann.send_response_broadcast().await;
        
        for iface in result_ann {
            match iface {
                Ok(_) => (),
                Err(msg) => 
                    error!("Failed to announce oneself on one of interfaces: {}", msg),
            }
        }
    });
    

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
                    info!("Received response packet from '{}'", addr);
                    debug!("{:?}", ether_rr.found_nodes());
                }

                Ok((NCNRPPacket::Heartbeat(hp), addr)) => {
                    debug!("Received heartbeat packet from '{addr}'");
                    
                    if !ether_rr.is_in_found_nodes(&hp.pub_key_hash) {
                        let ether_req = ether_rr.clone();
                        tokio::spawn(async move {
                            let delay = rand::random_range(10..=200);
                            tokio::time::sleep(time::Duration::from_millis(delay)).await;

                            if !ether_req.is_in_found_nodes(&hp.pub_key_hash) {
                                if let Err(msg) = ether_req.send_request_to(addr).await {
                                    error!("Failed to send request to '{addr}': {msg}");
                                }
                            }
                        });
                    }
                }
                
                Err(msg) => error!("Failed to receive packet: {}", msg),
            }
        }
    });

    // Task 2: garbage cleaner
        let ether_gc = ether.clone();
        tokio::spawn( async move {
            let mut tick = tokio::time::interval(time::Duration::from_secs(3u64));
            loop {
                tick.tick().await;
                if ether_gc.found_nodes_gc() > 0 {
                    debug!("Found nodes pool was cleared by GC");
                }
            }
        });

        let ether_hb = ether.clone();
        tokio::spawn(async move {
            loop {
                let (base_hb_interval, dynamic_ttl) = ether_hb.get_adaptive_timing();
                
                let jitter = std::time::Duration::from_millis(rand::random_range(16..=250));
                tokio::time::sleep(base_hb_interval + jitter).await;

                let results = ether_hb.send_heartbeat_broadcast_with_ttl(dynamic_ttl).await;
                for result in results {
                    if let Err(msg) = result {
                        error!("Failed to send heartbeat packet: {}", msg);
                    }
                }
                debug!("Sent heartbeat packet (nodes: {})", ether_hb.found_nodes().len());
            }
        });

    
    info!("NetCast Daemon is up!");
    
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    info!("Shutting down NetCast Daemon...");
}
