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
        txt_data.insert("pub_key".to_string(), self.pub_key_hash.clone());
        txt_data.insert("name".to_string(), self.name.clone());
        txt_data.insert("os".to_string(), self.os.clone());
        
        // let txt_properties = TxtProperties::from(txt_data);

        ServiceInfo::new(
            "_netcast._tcp.local",
            &instance_name,
            &host_name,
            "",
            self.port,
            txt_data,
        ).expect("Failed to create mDNS packet !")
    }
}

pub struct BeaconBroadcast {
    pub found_nodes: Vec<BeaconPacket>,
    mdns: ServiceDaemon,
    pub own_beacon_packet: BeaconPacket
}

impl BeaconBroadcast {
    pub fn new(node_id: String, pub_key_hex: String, name: String, os: String, port: u16) -> Result<Self, mdns_sd::Error> {
        // let new_mdns = ServiceDaemon::new()?;
        Ok(Self { 
            found_nodes: vec![], 
            mdns: ServiceDaemon::new()?, 
            own_beacon_packet: 
                BeaconPacket { 
                    node_id,
                    pub_key_hash: pub_key_hex,
                    name,
                    os,
                    port,
                } 
        })
    }
    pub fn found_nodes(&self) -> &Vec<BeaconPacket> {
        &self.found_nodes
    }

}