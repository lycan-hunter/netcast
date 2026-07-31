//TODO: Make all discovery on custom simple udp protocol, DO NOT USE SLOPOMDNS-SD
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;

pub struct BeaconPacket {
    pub node_id: String,
    pub pub_key_hash: String,
    pub name: String,
    pub os: String,
    pub port: u16,
}

impl BeaconPacket {
    pub fn into_service_info(&self) -> ServiceInfo {
        let instance_name = self.name.clone();

        let host_name = format!("{}.local.", self.node_id);

        let mut txt_data: HashMap <String, String> = HashMap::new();
        txt_data.insert("node_id".to_string(), self.node_id.clone());
        txt_data.insert("pub_key_hash".to_string(), self.pub_key_hash.clone());
        txt_data.insert("name".to_string(), self.name.clone());
        txt_data.insert("os".to_string(), self.os.clone());
        
        ServiceInfo::new(
            "_netcast._tcp.local.",
            &instance_name,
            &host_name,
            "",
            self.port,
            txt_data,
        ).expect("Failed to create mDNS packet !")
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

pub struct BeaconEther {
    found_nodes: Vec<BeaconPacket>,
    pub mdns: ServiceDaemon,
    pub own_bp: BeaconPacket
}

impl BeaconEther {
    pub fn new(bp: BeaconPacket) -> Result<Self, mdns_sd::Error> {
        // let new_mdns = ServiceDaemon::new()?;
        Ok(Self { 
            found_nodes: vec![], 
            mdns: ServiceDaemon::new()?,
            own_bp: bp 
            }
        )
    }
    pub fn found_nodes(&self) -> &Vec<BeaconPacket> {
        &self.found_nodes
    }

}