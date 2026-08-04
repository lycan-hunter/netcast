mod discovery;
mod crypto;

use std::thread;
use std::time::Duration;

use discovery::BeaconEther;
use discovery::packet::{NCNRPPacket, ResponsePayload};

fn main() {
    let handle_node_b = thread::spawn(|| {
        let mut payload_b = ResponsePayload::default();
        payload_b.name = String::from("Node-B");
        payload_b.os_family = String::from("linux");
        payload_b.pub_key_hash = [2u8; 32];
        payload_b.tcp_port = 7002;

        let mut node_b = BeaconEther::new(payload_b)
            .expect("Failed to initialize Node B");

        println!("[Node B] Listening for discovery requests on 0.0.0.0:31337...");

        match node_b.recv_packet() {
            Ok((NCNRPPacket::Request, src_addr)) => {
                println!("[Node B] Received Request from {}. Responding...", src_addr);
                
                if let Err(e) = node_b.send_response_to(src_addr) {
                    eprintln!("[Node B] Failed to send response: {}", e);
                }

                if let Err(e) = node_b.send_request_to(src_addr) {
                    eprintln!("[Node B] Failed to send reciprocal request: {}", e);
                }
            }
            Ok((other, src)) => println!("[Node B] Unexpected packet {:?} from {}", other, src),
            Err(e) => eprintln!("[Node B] Recv error: {}", e),
        }

        match node_b.recv_packet() {
            Ok((NCNRPPacket::Response(rp), src)) => {
                println!("[Node B] Discovered peer: '{}' from {}", rp.name, src);
            }
            Ok((other, src)) => println!("[Node B] Unexpected packet {:?} from {}", other, src),
            Err(e) => eprintln!("[Node B] Recv error: {}", e),
        }

        println!("[Node B] Final discovered peers: {:#?}", node_b.found_nodes());
    });

    let handle_node_a = thread::spawn(|| {
        thread::sleep(Duration::from_millis(150));

        let mut payload_a = ResponsePayload::default();
        payload_a.name = String::from("Node-A");
        payload_a.os_family = String::from("macos");
        payload_a.pub_key_hash = [1u8; 32];
        payload_a.tcp_port = 7001;

        let mut node_a = BeaconEther::new_with_port(payload_a, 31338)
            .expect("Failed to initialize Node A");

        println!("[Node A] Sending Discovery Request Broadcast...");
        let _ = node_a.send_request_broadcast();

        match node_a.recv_packet() {
            Ok((NCNRPPacket::Response(rp), src)) => {
                println!("[Node A] Discovered peer: '{}' from {}", rp.name, src);
            }
            Ok((other, src)) => println!("[Node A] Unexpected packet {:?} from {}", other, src),
            Err(e) => eprintln!("[Node A] Recv error: {}", e),
        }

        match node_a.recv_packet() {
            Ok((NCNRPPacket::Request, src_addr)) => {
                println!("[Node A] Got reciprocal Request from {}. Replying...", src_addr);
                if let Err(e) = node_a.send_response_to(src_addr) {
                    eprintln!("[Node A] Failed to send response: {}", e);
                }
            }
            Ok((other, src)) => println!("[Node A] Unexpected packet {:?} from {}", other, src),
            Err(e) => eprintln!("[Node A] Recv error: {}", e),
        }

        println!("[Node A] Final discovered peers: {:#?}", node_a.found_nodes());
    });

    handle_node_b.join().unwrap();
    handle_node_a.join().unwrap();

}