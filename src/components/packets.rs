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

pub struct Packet {
    pub packet_id: PacketId,
    pub payload: Option<Vec<u8>>,
}

// ================================================
// Implementation
// ================================================
impl PacketId {
    pub fn from_u8(num: u8) -> PacketId {
        match num {
            0 => PacketId::Heartbeat,
            1 => PacketId::HeartbeatAck,
            2 => PacketId::RequestSendReplica,
            3 => PacketId::SendReplica,
            4 => PacketId::SendReplicaAck,
            5 => PacketId::AskIp,
            6 => PacketId::AskIpAck,
            7 => PacketId::RequestFromClient,
            8 => PacketId::ResponseNodeIp,
            9 => PacketId::ClientUpload,
            10 => PacketId::DataNodeSendData,
            11 => PacketId::ClientRequestAck,
            12 => PacketId::StateSync,
            13 => PacketId::StateSyncAck,
            14 => PacketId::Notify,
            _ => panic!("Incorrect PacketId: got {num}",),
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
        }
    }

    /// Create Packet from received bytes
    pub fn from_bytes(bytes: &[u8]) -> Packet {
        assert!(bytes.len() >= 5, "Length of bytes not greater than 5");
        let packet_id = PacketId::from_u8(bytes[0]);
        let payload_size = u32::from_be_bytes(bytes[1..5].try_into().expect("Incorrect length"));
        assert!(
            bytes.len() == (5 + payload_size) as usize,
            "Length of bytes not equal {} ( = 1 + 4 + {})",
            5 + payload_size,
            payload_size
        );
        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&bytes[5..(5 + payload_size) as usize]);
        Packet {
            packet_id,
            payload: Some(payload),
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
