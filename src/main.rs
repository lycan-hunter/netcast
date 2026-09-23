mod discovery;
// mod crypto;
mod channel;
mod crypto;
mod ui;

use log::{error, warn, info, debug, trace};
use rand;
use env_logger::{Builder, Env, WriteStyle};
use std::time;
use std::sync::Arc;

use ui::DaemonEther;

use discovery::BeaconEther;
use discovery::packet::{NCNRPPacket, ResponsePayload};

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// async fn handle_client(&uether) -> std::io::Result<()> {
        

#[tokio::main]
async fn main() {
    // --- Logger init ---
    Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None) 
        .format_module_path(false) 
        .write_style(WriteStyle::Auto) 
        .init();


    // ######################
    // # Network ether init #
    // ######################

    let own_discovery_packet = ResponsePayload::new(
        900, 
        8086, 
        String::from("darwin-aarch64-macosx"), 
        [1u8; 32],
    String::from("NetCast's-Node612A"), 
    );

    let network_ether: Arc<BeaconEther>;
    loop {
        match BeaconEther::new_with_port(own_discovery_packet.clone(), 31338).await {
            Err(msg) => {
                error!("Failed to start netcast interface: {}", msg);
                info!("Retrying to start netcast interface...");
            },
            Ok(be) => {
                network_ether = Arc::new(be);
                break;
            }
        }
        tokio::time::sleep(time::Duration::from_secs(3)).await;
    }

    // First broadcast announcement
    let network_ether_ann = network_ether.clone();
    tokio::spawn(async move {
        let result_ann = network_ether_ann.send_response_broadcast().await;
        
        for iface in result_ann {
            match iface {
                Ok(_) => (),
                Err(msg) => 
                    error!("Failed to announce oneself on one of interfaces: {}", msg),
            }
        }
    });
    

    // Task 1: recieving/responsing
    let network_ether_rr = network_ether.clone();
    tokio::spawn( async move {
        
        loop {
            match network_ether_rr.recv_packet().await {

                Ok((NCNRPPacket::Request, addr)) => {

                    tokio::time::sleep(time::Duration::from_millis(rand::random_range(16..=120))).await;

                    match network_ether_rr.send_response_to(addr).await {
                        Err(msg) => error!("Failed to send response: {}", msg),
                        Ok(_) => (),
                    }
                }

                Ok((NCNRPPacket::Response(_), addr)) => {
                    info!("Received response packet from '{}'", addr);
                    debug!("{:?}", network_ether_rr.found_nodes());
                }

                Ok((NCNRPPacket::Heartbeat(hp), addr)) => {
                    debug!("Received heartbeat packet from '{addr}'");
                    
                    if !network_ether_rr.is_in_found_nodes(&hp.pub_key_hash) {
                        let ether_req = network_ether_rr.clone();
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
    let network_ether_gc = network_ether.clone();
    tokio::spawn( async move {
        let mut tick = tokio::time::interval(time::Duration::from_secs(3u64));
        loop {
            tick.tick().await;
            if network_ether_gc.found_nodes_gc() > 0 {
                debug!("Found nodes pool was cleared by GC");
            }
        }
    });
    
    // Task 3: Sending hearbeat packets
    let network_ether_hb = network_ether.clone();
    tokio::spawn(async move {
        loop {
            let (base_hb_interval, dynamic_ttl) = network_ether_hb.get_adaptive_timing();
            
            let jitter = std::time::Duration::from_millis(rand::random_range(16..=250));
            tokio::time::sleep(base_hb_interval + jitter).await;

            let results = network_ether_hb.send_heartbeat_broadcast_with_ttl(dynamic_ttl).await;
            for result in results {
                if let Err(msg) = result {
                    error!("Failed to send heartbeat packet: {}", msg);
                }
            }
            debug!("Sent heartbeat packet (nodes: {})", network_ether_hb.found_nodes().len());
        }
    });

    // ##############################
    // # Unix socket interface init #
    // ##############################

    let unisock_ether: Arc<DaemonEther>;
    loop {
        match DaemonEther::new(network_ether.clone()).await {
            Err(msg) => {
                error!("Failed to start netcast unix socket: {}", msg);
                info!("Retrying to start netcast unix socket...");
            },
            Ok(de) => {
                unisock_ether = Arc::new(de);
                break;
            }
        }
        tokio::time::sleep(time::Duration::from_secs(3)).await;
    }

    let unix_ether = unisock_ether.clone();
    

    info!("NetCast Daemon is up!");
    
    tokio::signal::ctrl_c().await.expect("Aborted by SIGTERM");
    info!("Shutting down NetCast Daemon...");
}
