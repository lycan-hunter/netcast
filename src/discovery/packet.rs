use std::io;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NCNRPPacket {
    Request = 0x01,
    Response(ResponsePayload) = 0x02,
    Heartbeat(HeartbeatPayload) = 0x03,
}

impl NCNRPPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"NC");
        buf.push(1u8); // Version

        match self {
            NCNRPPacket::Request => {
                buf.push(0x01); // Type 0x01
            },
            NCNRPPacket::Response(payload) => {
                buf.push(0x02); // Type 0x02
                let payload_bytes = payload.serialize();
                buf.extend_from_slice(&(payload_bytes.len() as u32).to_be_bytes());
                buf.extend_from_slice(&payload_bytes);
            }
            NCNRPPacket::Heartbeat(payload) => {
                buf.push(0x03); // Type 0x03
                let payload_bytes = payload.serialize();
                buf.extend_from_slice(&payload_bytes);
            }
        }
        buf
    }

    pub fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 4 || bytes.len() > 812 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad packet size"));
        }
        if &bytes[0..2] != b"NC" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }
        if bytes[2] != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        let packet_type = bytes[3];

        match packet_type {
            0x01 => {
                if bytes.len() != 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Request packet must be exactly 4 bytes"));
                }
                Ok(NCNRPPacket::Request)
            }
            0x02 => {
                let payload_len = u32::from_be_bytes(
                    bytes.get(4..8)
                        .and_then(|b| b.try_into().ok())
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Truncated length field"))?
                ) as usize;

                if bytes.len() != 8 + payload_len {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet size mismatch with header length"));
                }

                let payload = ResponsePayload::deserialize(&bytes[8..])?;
                Ok(NCNRPPacket::Response(payload))
            }
            0x03 => {
                if bytes.len() != 38 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Heartbeat packet must be exactly 38 bytes"));
                }
                
                let payload = HeartbeatPayload::deserialize(&bytes[4..])?;
                Ok(NCNRPPacket::Heartbeat(payload))
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown packet type")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsePayload {
    pub ttl_seconds: u16,
    pub tcp_port: u16,
    pub os_family: String,
    pub pub_key_hash: [u8; 32],
    pub name: String,
}

impl ResponsePayload {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&self.ttl_seconds.to_be_bytes());
        buf.extend_from_slice(&self.tcp_port.to_be_bytes());
        buf.extend_from_slice(&self.pub_key_hash);

        for s in [&self.os_family, &self.name] {
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

        let mut read_str = || -> io::Result<String> {
            let len = *read_bytes(1)?.first().unwrap() as usize;
            let str_bytes = read_bytes(len)?;
            String::from_utf8(str_bytes.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        };

        let os_family = read_str()?;
        let name = read_str()?;

        if pos != bytes.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Extra garbage or size mismatch"));
        }

        Ok(Self { ttl_seconds, tcp_port, os_family, pub_key_hash, name })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatPayload {
    pub ttl_seconds: u16,
    pub pub_key_hash: [u8; 32],
}

impl HeartbeatPayload {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&self.ttl_seconds.to_be_bytes());
        buf.extend_from_slice(&self.pub_key_hash);
        buf
    }

    fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != 34 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Heartbeat size"));
        }
        let ttl_seconds = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        let mut pub_key_hash = [0u8; 32];
        pub_key_hash.copy_from_slice(&bytes[2..34]);

        Ok(Self { ttl_seconds, pub_key_hash })
    }
}
