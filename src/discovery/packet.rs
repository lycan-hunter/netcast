use std::io;

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
    pub fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 8 || bytes.len() > 812 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad packet size"));
        }
        if &bytes[0..2] != b"NC" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }
        if bytes[2] != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        let packet_type = bytes[3];
        let payload_len = u32::from_be_bytes(
            bytes.get(4..8)
                .and_then(|b| b.try_into().ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Truncated length"))?
        ) as usize;

        match packet_type {
            0x01 => {
                if payload_len != 0 || bytes.len() != 8 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Request must be empty"));
                }
                Ok(NCNRPPacket::Request)
            }
            0x02 => {
                let start = 8;
                let end = start + payload_len;
                let payload_bytes = bytes.get(start..end)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Payload truncated"))?;

                let payload = ResponsePayload::deserialize(payload_bytes)?;
                Ok(NCNRPPacket::Response(payload))
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown packet type")),
        }
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

        for s in [&self.os_family, &self.node_id, &self.name] {
            let bytes = s.as_bytes();
            let len = bytes.len().min(255) as u8;
            buf.push(len);
            buf.extend_from_slice(&bytes[..len as usize]);
        }

        buf
    }

    fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        let mut pos = 0;

        let mut read_bytes = |len: usize| -> io::Result<&[u8]> {
            let res = bytes.get(pos..pos + len)
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Data truncated"))?;
            pos += len;
            Ok(res)
        };

        let ttl_seconds = u16::from_be_bytes(read_bytes(2)?.try_into().unwrap());
        let tcp_port = u16::from_be_bytes(read_bytes(2)?.try_into().unwrap());
        
        let mut pub_key_hash = [0u8; 32];
        pub_key_hash.copy_from_slice(read_bytes(32)?);

        let mut read_str = |pos: &mut usize| -> io::Result<String> {
            let len = *bytes.get(*pos)
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "String len missing"))? as usize;
            *pos += 1;

            let str_bytes = bytes.get(*pos..*pos + len)
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "String truncated"))?;
            *pos += len;

            String::from_utf8(str_bytes.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        };

        let os_family = read_str(&mut pos)?;
        let node_id = read_str(&mut pos)?;
        let name = read_str(&mut pos)?;

        if pos != bytes.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Extra garbage or size mismatch"));
        }

        Ok(Self { ttl_seconds, tcp_port, os_family, pub_key_hash, node_id, name })
    }
}


//  #####################################
//  #   Request Packet Specification    #
//  #####################################

// Byte 0 ......... Byte 1 ........ Byte 2 ........ Byte 3 ........
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |      'N'      |      'C'      |  Version (1)  |  Type (0x01)  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                       Payload Length (0)                      |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+


//  ######################################
//  #   Response Packet Specification    #
//  ######################################

// Byte 0 ......... Byte 1 ........ Byte 2 ........ Byte 3 ........
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |      'N'      |      'C'      |  Version (1)  |  Type (0x02)  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                         Payload Length                        |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
//                Response Payload (struct is below)
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+


//  #######################################
//  #   Response Payload Specification    #
//  #######################################

// Byte 0 ......... Byte 1 ........ Byte 2 ........ Byte 3 ........
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |          TTL Seconds          |           TCP Port            |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
// +                                                               +
// |                    Public Key Hash (32 bytes)                 |
// +                                                               +
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | OS Fam Length | OS Family String  (variable length, max 255)  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | Node ID Length| Node ID String    (variable length, max 255)  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |  Name Length  | Name String       (variable length, max 255)  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
