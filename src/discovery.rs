mod discovery {

use mdns_sd::ServiceInfo;
use std::collections::HashMap;


pub struct BeaconPacket {
    pub node_id: String,
    pub pub_key_hex: String,
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
        txt_data.insert("pub_key".to_string(), self.pub_key_hex.clone());
        txt_data.insert("name".to_string(), self.name.clone());
        txt_data.insert("os".to_string(), self.os.clone());
        
        // let txt_properties = TxtProperties::from(txt_data);

        ServiceInfo::new(
            "_netcast._tcp.local.",
            &instance_name,
            &host_name,
            "",
            self.port,
            txt_data,
        ).expect("Failed to create mDNS packet !")
    }
}

pub struct BeaconBroadcast {
    found_nodes: Vec<BeaconPacket>,
}

impl BeaconBroadcast {
    pub fn found_nodes(&self) -> &Vec<BeaconPacket> {
        &self.found_nodes
    }

    pub fn send(bp: &BeaconPacket){

    }
    pub fn scan(&self){

    }
}

}