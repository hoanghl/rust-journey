use crate::components::address::Address;
use crate::components::errors::ParseError;

// ================================================
// Definition for enum and constants
// ================================================

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PacketId {
    Heartbeat = 0,
    HeartbeatAck = 1,
    RequestSendReplica = 2,
    SendReplica = 3,
    SendReplicaAck = 4,
    AskIp = 5,
    AskIpAck = 6,
    RequestFromClient = 7,
    ResponseNodeIp = 8,
    ClientUpload = 9,
    DataNodeSendData = 10,
    ClientRequestAck = 11,
    StateSync = 12,
    StateSyncAck = 13,
    Notify = 14,
}

pub enum Request {
    Download = 0,
    Upload = 1,
}

pub const BYTE_SEP_CHARACTER: u8 = 124; // byte value of character '|'
pub const SIZE_HEADER: usize = 5;

pub struct Packet {
    pub packet_id: PacketId,
    pub payload: Option<Vec<u8>>,

    pub addr_sender: Option<Address>,
    pub addr_receiver: Option<Address>,
}

// ================================================
// Implementation
// ================================================

impl PacketId {
    pub fn from_u8(num: u8) -> Result<PacketId, ParseError> {
        match num {
            0 => Ok(PacketId::Heartbeat),
            1 => Ok(PacketId::HeartbeatAck),
            2 => Ok(PacketId::RequestSendReplica),
            3 => Ok(PacketId::SendReplica),
            4 => Ok(PacketId::SendReplicaAck),
            5 => Ok(PacketId::AskIp),
            6 => Ok(PacketId::AskIpAck),
            7 => Ok(PacketId::RequestFromClient),
            8 => Ok(PacketId::ResponseNodeIp),
            9 => Ok(PacketId::ClientUpload),
            10 => Ok(PacketId::DataNodeSendData),
            11 => Ok(PacketId::ClientRequestAck),
            12 => Ok(PacketId::StateSync),
            13 => Ok(PacketId::StateSyncAck),
            14 => Ok(PacketId::Notify),
            _ => Err(ParseError::incorrect_packet_id(num)),
        }
    }
}

impl Packet {
    /// Creates Packet from already known packetId and payload.
    /// # Example
    /// ```rust
    /// let packet = Packet.new(PacketId::Heartbeat, None)
    /// ```
    pub fn new(packet_id: PacketId, payload: Option<&[u8]>) -> Packet {
        // TODO: HoangLe [Apr-28]: Modify this to create different packets
        let payload_field: Option<Vec<u8>> = match payload {
            Some(payload) => {
                let mut vec = Vec::<u8>::new();
                vec.extend_from_slice(payload);
                Some(vec)
            }
            None => None,
        };
        Packet {
            packet_id,
            payload: payload_field,
            addr_sender: None,
            addr_receiver: None,
        }
    }

    /// Extract packet to byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        // Add packet ID
        bytes.push(self.packet_id as u8);

        // Add payload size
        let mut payload_size = 0;
        if self.payload != None {
            payload_size = self.payload.as_ref().unwrap().len();
        }
        let bytes_payload_size = (payload_size as u32).to_be_bytes();
        bytes.extend_from_slice(&bytes_payload_size);

        // Add payload
        if self.payload != None {
            bytes.extend_from_slice(self.payload.as_ref().unwrap());
        }

        return bytes;
    }

    // ================================================
    // Functions to create packets
    // ================================================

    /// Create Packet from received bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Packet, ParseError> {
        if bytes.len() < SIZE_HEADER {
            return Err(ParseError::incorrect_min_header_size(bytes.len()));
        }

        let packet_id = match PacketId::from_u8(bytes[0]) {
            Ok(packet_id) => packet_id,
            Err(e) => return Err(e),
        };

        let payload_size = u32::from_be_bytes(bytes[1..5].try_into().expect("Incorrect length")) as usize;
        if bytes.len() != SIZE_HEADER + payload_size {
            return Err(ParseError::mismatched_packet_size(bytes.len(), payload_size));
        }

        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&bytes[5..(5 + payload_size) as usize]);

        Ok(Packet {
            packet_id,
            payload: Some(payload),
            addr_receiver: None,
            addr_sender: None,
        })
    }

    pub fn create_heartbeat() -> Packet {
        Packet {
            packet_id: PacketId::Heartbeat,
            payload: None,
            addr_receiver: None,
            addr_sender: None,
        }
    }
    pub fn create_heartbeat_ack() -> Packet {
        Packet {
            packet_id: PacketId::HeartbeatAck,
            payload: None,
            addr_receiver: None,
            addr_sender: None,
        }
    }
    // pub fn create_RequestSendReplica() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_SendReplica() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_SendReplicaAck() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_AskIp() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_AskIpAck() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_RequestFromClient() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_ResponseNodeIp() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_ClientUpload() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_DataNodeSendData() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_ClientRequestAck() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_StateSync() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_StateSyncAck() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
    // pub fn create_Notify() -> Packet {
    //     // TODO: HoangLe [Apr-28]: Implement this
    // }
}

impl std::fmt::Display for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PacketId::Heartbeat => "Heartbeat",
            PacketId::HeartbeatAck => "HeartbeatAck",
            PacketId::RequestSendReplica => "RequestSendReplica",
            PacketId::SendReplica => "SendReplica",
            PacketId::SendReplicaAck => "SendReplicaAck",
            PacketId::AskIp => "AskIp",
            PacketId::AskIpAck => "AskIpAck",
            PacketId::RequestFromClient => "RequestFromClient",
            PacketId::ResponseNodeIp => "ResponseNodeIp",
            PacketId::ClientUpload => "ClientUpload",
            PacketId::DataNodeSendData => "DataNodeSendData",
            PacketId::ClientRequestAck => "ClientRequestAck",
            PacketId::StateSync => "StateSync",
            PacketId::StateSyncAck => "StateSyncAck",
            PacketId::Notify => "Notify",
        };
        write!(f, "{}", s)
    }
}
