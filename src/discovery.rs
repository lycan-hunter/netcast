// TODO: Make recv methods 
use std::collections::HashMap;
use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};
use pnet_datalink;

pub mod packet;
use packet::{ResponsePayload, NCNRPPacket};

pub const NETCAST_PORT: u16 = 31337;

// NetCast Node Resolution Protocol
pub struct BeaconEther {
    found_nodes: Vec<ResponsePayload>,
    sock: UdpSocket,
    pub own_bp: NCNRPPacket,
    interfaces: Vec<(Ipv4Addr, Ipv4Addr)>, // INTERFACE_IP [0], BROADCAST_IP [1]
    buf: [u8; 812]
}

impl BeaconEther {
    pub fn new(bp: ResponsePayload) -> Result<Self, std::io::Error> {
        let sock = UdpSocket::bind(("0.0.0.0", NETCAST_PORT))?;
        sock.set_broadcast(true)?;
        Ok(Self { 
            found_nodes: vec![], 
            sock: sock,
            own_bp: NCNRPPacket::Response(bp),
            interfaces: BeaconEther::get_all_bcast_targets(),
            }
        )
    }

    pub fn send_request_broadcast(&self) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(&NCNRPPacket::Request.serialize(),
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)));
        }
        ret

    }

    pub fn send_request_to(&self, ip: SocketAddrV4) -> std::io::Result<usize> {
        self.sock.send_to(&NCNRPPacket::Request.serialize(), ip)
    }


    pub fn send_response_broadcast(&self) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(&self.own_bp.serialize(),
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)));
        }
        ret

    } 

    pub fn send_response_to(&self, ip: SocketAddrV4) -> std::io::Result<usize> {
        self.sock.send_to(&self.own_bp.serialize(), ip)
    }

    pub fn recv_packet(& mut self) -> std::io::Result<(NCNRPPacket, SocketAddrV4)> {
        let (byte_len, sock_dst) = self.sock.recv_from(&mut self.buf)?;
        let packet = NCNRPPacket::deserialize(&self.buf[..byte_len])?;
        if let NCNRPPacket::Response(rp) = &packet {
            self.add_found_node(rp.clone());
        }
        if let std::net::SocketAddr::V4(v4_addr) = sock_dst {
            Ok((packet, v4_addr))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "IPv6 addr not supported yet"))
        }
        

    }

    pub fn found_nodes(&self) -> &Vec<ResponsePayload> {
        &self.found_nodes
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

    fn add_found_node(&mut self, rp: ResponsePayload) {
        if let Some(existing) = self.found_nodes.iter_mut().find(|n| n.pub_key_hash == rp.pub_key_hash) {
            *existing = rp;
        } else {
            self.found_nodes.push(rp);
        }
    }

}