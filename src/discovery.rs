// TODO: make own protocol
use std::collections::HashMap;
use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};
use pnet_datalink;

pub mod packet;

pub const NETCAST_PORT: u16 = 31337;

pub struct BeaconPacket {
    pub node_id: String,
    pub pub_key_hash: String,
    pub name: String,
    pub os: String,
    pub port: u16,
}

impl BeaconPacket {
    pub fn into_service_info(&self) {
        let instance_name = self.name.clone();

        let host_name = format!("{}.local.", self.node_id);

        let mut txt_data: HashMap <String, String> = HashMap::new();
        txt_data.insert("node_id".to_string(), self.node_id.clone());
        txt_data.insert("pub_key_hash".to_string(), self.pub_key_hash.clone());
        txt_data.insert("name".to_string(), self.name.clone());
        txt_data.insert("os".to_string(), self.os.clone());
    }

    pub fn new() -> Self {
        Self {
            node_id: String::from(""),
            pub_key_hash: String::from(""),
            name: String::from(""),
            os: String::from(""),
            port: 0,
            
        }
    }
}

// NetCast Node Resolution Protocol
pub struct BeaconEther {
    found_nodes: Vec<BeaconPacket>,
    sock: UdpSocket,
    pub own_bp: BeaconPacket,
    interfaces: Vec<(Ipv4Addr, Ipv4Addr)>, // INTERFACE_IP [0], BROADCAST_IP [1]
}

impl BeaconEther {
    pub fn new(bp: BeaconPacket) -> Result<Self, std::io::Error> {
        let sock = UdpSocket::bind(("0.0.0.0", NETCAST_PORT))?;
        // sock.set_broadcast(true)?;
        sock.set_broadcast(true)?;
        Ok(Self { 
            found_nodes: vec![], 
            sock: sock,
            own_bp: bp,
            interfaces: BeaconEther::get_all_bcast_targets(),
            }
        )
    }

    pub fn send_beacon_broadcast(&self, payload: &[u8]) -> Vec<std::io::Result<usize>> {
        let mut ret: Vec<std::io::Result<usize>> = vec![];
        for ip_bcast in &self.interfaces {
            ret.push(self.sock
                .send_to(payload,
                     SocketAddrV4::new(ip_bcast.1, NETCAST_PORT)));
        }
        ret

    }

    pub fn recv_beacon(&self, buf: &mut [u8]) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.sock.recv_from(buf)
    }

    pub fn found_nodes(&self) -> &Vec<BeaconPacket> {
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

}