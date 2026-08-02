pub const NC_MAGIC_NUM: u16 = 0x4E43;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum NCNRPPacket {
    Request = 0x01,
    Response(ResponsePayload) = 0x02,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsePayload {
    pub ttl_seconds: u16,
    pub tcp_port: u16,
    pub os: String,
    pub pub_key_hash: [u8; 32],
    pub node_id: String,
    pub display_name: String,
}