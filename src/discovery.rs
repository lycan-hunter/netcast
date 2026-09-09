use std::net::{Ipv4Addr, SocketAddrV4};
use std::time;
use pnet_datalink;
use tokio::net::UdpSocket;
use dashmap::DashMap;

pub mod packet;
use packet::{ResponsePayload, NCNRPPacket};

use crate::discovery::packet::HeartbeatPayload;


pub const NETCAST_PORT: u16 = 31337;

// NetCast Node Resolution Protocol
struct FoundNodeInfo {
    payload: ResponsePayload,
    deadline: time::Instant,
}

pub struct BeaconEther {
    found_nodes: dashmap::DashMap<[u8;32], FoundNodeInfo>,
    sock: UdpSocket,
    pub own_bp: NCNRPPacket,
    own_hbp: NCNRPPacket,
    interfaces: Vec<(Ipv4Addr, Ipv4Addr)>, // INTERFACE_IP [0], BROADCAST_IP [1]
}

impl BeaconEther {
    pub async fn new_with_port(bp: ResponsePayload, port: u16) -> Result<Self, std::io::Error> {
        let sock = UdpSocket::bind(("0.0.0.0", port)).await?;
        sock.set_broadcast(true)?;
        Ok(Self { 
            found_nodes: DashMap::new(), 
            sock,
            own_bp: NCNRPPacket::Response(bp.clone()),
            own_hbp: NCNRPPacket::Heartbeat(HeartbeatPayload {
                 ttl_seconds: 15, pub_key_hash: bp.pub_key_hash }),
            interfaces: BeaconEther::get_all_bcast_targets(),
            }
        )
    }

    pub async fn new(bp: ResponsePayload) -> Result<Self, std::io::Error> {
        BeaconEther::new_with_port(bp, NETCAST_PORT).await
    }

    pub async fn send_request_broadcast(&self) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(&NCNRPPacket::Request.serialize(),
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)).await);
        }
        ret

    }

    pub async fn send_request_to(&self, ip: SocketAddrV4) -> std::io::Result<usize> {
        self.sock.send_to(&NCNRPPacket::Request.serialize(), ip).await
    }


    pub async fn send_response_broadcast(&self) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(&self.own_bp.serialize(),
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)).await);
        }
        ret

    } 

    pub async fn send_response_to(&self, ip: SocketAddrV4) -> std::io::Result<usize> {
        self.sock.send_to(&self.own_bp.serialize(), ip).await
    }

    pub async fn recv_packet(&self) -> std::io::Result<(NCNRPPacket, SocketAddrV4)> {
        loop {
            let mut buf = [0u8; 812];
            let (byte_len, sock_dst) = self.sock.recv_from(&mut buf).await?;
            let packet = NCNRPPacket::deserialize(&buf[..byte_len])?;
            
            let own_pkh = match &self.own_bp {
                NCNRPPacket::Response(resp_payload) => resp_payload.pub_key_hash,
                _ => [0u8; 32],
            };

            let is_own_packet = match &packet {
                NCNRPPacket::Response(rp) => rp.pub_key_hash == own_pkh,
                NCNRPPacket::Heartbeat(hp) => hp.pub_key_hash == own_pkh,
                NCNRPPacket::Request => false,
            };

            if is_own_packet {
                continue;
            }
            
            match &packet {
                NCNRPPacket::Response(rp) => {
                    self.add_found_node(rp.clone());
                },
                NCNRPPacket::Heartbeat(hp) => {
                    if hp.pub_key_hash != own_pkh {
                        if let Some(mut node) = self.found_nodes.get_mut(&hp.pub_key_hash) {
                            node.deadline = time::Instant::now() + time::Duration::from_secs(hp.ttl_seconds as u64);
                        }
                    }
                },
                _ => {},
            }
            if let std::net::SocketAddr::V4(v4_addr) = sock_dst {
                return Ok((packet, v4_addr))
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "IPv6 addr not supported yet"))
            }
        }

    }

    pub async fn send_heartbeat_broadcast(&self) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(&self.own_hbp.serialize(),
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)).await);
        }
        ret
    }

    pub fn found_nodes_gc(&self) {
        let now = time::Instant::now();
        self.found_nodes.retain(|_key, node| node.deadline > now);
    }


    pub fn found_nodes(&self) -> Vec<ResponsePayload> {
        
        let now = time::Instant::now();
        
        self.found_nodes.iter()
            .filter(|entry| entry.value().deadline > now)
            .map(|entry| entry.value().payload.clone())
            .collect()
    }

    pub fn is_in_found_nodes(&self, pkh: &[u8; 32]) -> bool {
        if let Some(_) = self.found_nodes.get(pkh) {
            true
        } else {
            false
        }
    }

    pub fn get_all_bcast_targets() -> Vec<(Ipv4Addr, Ipv4Addr)> {
        let mut targets = Vec::new();

        for iface in pnet_datalink::interfaces() {
            if !iface.is_up() || iface.is_loopback() {
                continue;
            }

            for ip_net in iface.ips {
                if let (std::net::IpAddr::V4(ipv4), std::net::IpAddr::V4(bcast)) = (ip_net.ip(), ip_net.broadcast()) {
                    targets.push((ipv4, bcast));
                }
            }
        }

        targets
    }

    fn add_found_node(&self, rp: ResponsePayload) {
            let deadline = time::Instant::now() + time::Duration::from_secs(rp.ttl_seconds as u64);
            let key = rp.pub_key_hash;

            let entry = FoundNodeInfo {
                payload: rp,
                deadline,
            };
            
            self.found_nodes.insert(key, entry);
    }

}