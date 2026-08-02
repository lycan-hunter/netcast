#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NCNRPPacket {
    Request = 0x01,
    Response(ResponsePayload) = 0x02,
}

impl NCNRPPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"NC");
        buf.push(1u8);

        match self {
            NCNRPPacket::Request => {
                buf.push(0x01);
                buf.extend_from_slice(&0u32.to_be_bytes());
            },
            NCNRPPacket::Response(payload) => {
                buf.push(0x02);

                let payload_bytes = payload.serialize();
                buf.extend_from_slice(&(payload_bytes.len() as u32).to_be_bytes());
                buf.extend_from_slice(&payload_bytes);
            }
        }

        buf

    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsePayload {
    // ALL "String" 255 BYTES LENGHT MAX !
    pub ttl_seconds: u16,
    pub tcp_port: u16,
    pub os_family: String,
    pub pub_key_hash: [u8; 32],
    pub node_id: String,
    pub name: String,
}

impl ResponsePayload {
    pub fn new() -> Self {
        Self { 
            ttl_seconds: 900,
            tcp_port: 0,
            os_family: String::from(""),
            pub_key_hash: [0u8; 32],
            node_id: String::from(""), 
            name: String::from("")
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        // -- Static length fields --

        buf.extend_from_slice(&self.ttl_seconds.to_be_bytes());
        buf.extend_from_slice(&self.tcp_port.to_be_bytes());
        buf.extend_from_slice(&self.pub_key_hash);

        // -- Dynamic length fields --

        let os_bytes = self.os_family.as_bytes();
        let os_len = self.os_family.len().min(255) as u8;

        buf.push(os_len);
        buf.extend_from_slice(os_bytes);

        let node_bytes = self.node_id.as_bytes();
        let node_len = self.node_id.len().min(255) as u8;

        buf.push(node_len);
        buf.extend_from_slice(node_bytes);

        let name_bytes = self.name.as_bytes();
        let name_len = self.name.len().min(255) as u8;

        buf.push(name_len);
        buf.extend_from_slice(name_bytes);

        buf
    }
}