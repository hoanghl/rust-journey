use std::convert::From;
use std::fmt::{self};
use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr, TcpStream},
};

use crate::components::errors::ParseError;

// ================================================
// Definition for enum and constants
// ================================================

const BYTE_SEP_CHARACTER: u8 = 124; // byte value of character '|'
const SIZE_HEADER: usize = 5;
const BUFF_LEN: usize = 1024;

#[rustfmt::skip]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PacketId {
    Default                 = 0,
    Heartbeat               = 1,
    HeartbeatAck            = 2,
    RequestSendReplica      = 3,
    SendReplica             = 4,
    SendReplicaAck          = 5,
    AskIp                   = 6,
    AskIpAck                = 7,
    RequestFromClient       = 8,
    ResponseNodeIp          = 9,
    ClientUpload            = 10,
    DataNodeSendData        = 11,
    ClientRequestAck        = 12,
    StateSync               = 13,
    StateSyncAck            = 14,
    Notify                  = 15,
}

pub struct Packet {
    // General attributes
    pub packet_id: PacketId,
    pub addr_sender: Option<SocketAddr>,
    pub addr_receiver: Option<SocketAddr>,

    // Attributes dedicated for sending
    pub payload: Option<Vec<u8>>,

    // Attributed parsed while receiving
    pub ip_master: Option<Ipv4Addr>,
}

// ================================================
// Implementation
// ================================================

impl From<u8> for PacketId {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketId::Heartbeat,
            1 => PacketId::Heartbeat,
            2 => PacketId::HeartbeatAck,
            3 => PacketId::RequestSendReplica,
            4 => PacketId::SendReplica,
            5 => PacketId::SendReplicaAck,
            6 => PacketId::AskIp,
            7 => PacketId::AskIpAck,
            8 => PacketId::RequestFromClient,
            9 => PacketId::ResponseNodeIp,
            10 => PacketId::ClientUpload,
            11 => PacketId::DataNodeSendData,
            12 => PacketId::ClientRequestAck,
            13 => PacketId::StateSync,
            14 => PacketId::StateSyncAck,
            15 => PacketId::Notify,
            _ => panic!("Error as parsing to enum PacketId: value = {}", value),
        }
    }
}

impl std::fmt::Display for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PacketId::Default => "Default",
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

impl std::fmt::Debug for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PacketId::Default => "Default",
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

impl Default for Packet {
    fn default() -> Self {
        Packet {
            packet_id: PacketId::Default,
            payload: None,
            addr_sender: None,
            addr_receiver: None,
            ip_master: None,
        }
    }
}

impl Packet {
    /// Creates Packet from already known packetId and payload.
    /// # Example
    /// ```rust
    /// let packet = Packet.new(PacketId::Heartbeat, None)
    /// ```
    pub fn new(packet_id: PacketId, payload: Option<Vec<u8>>) -> Packet {
        Packet {
            packet_id,
            payload,
            ..Default::default()
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

        bytes
    }

    // ================================================
    // Functions to create packets
    // ================================================

    /// Create Packet from stream
    pub fn from_stream(stream: &mut TcpStream) -> Result<Packet, ParseError> {
        log::debug!("Receive data from: {}", stream.peer_addr().unwrap());

        // ================================================
        // Read bytes from stream
        // ================================================
        let mut bytes = Vec::<u8>::new();
        let mut buff: [u8; BUFF_LEN] = [0; BUFF_LEN];
        loop {
            let n = stream.read(&mut buff).unwrap();
            if n == 0 {
                break;
            } else {
                bytes.extend_from_slice(&buff[0..n]);

                if n < BUFF_LEN {
                    break;
                }
            }
        }

        log::debug!("Total bytes received: {}", bytes.len());

        // ================================================
        // Parse header
        // ================================================
        if bytes.len() < SIZE_HEADER {
            return Err(ParseError::incorrect_min_header_size(bytes.len()));
        }

        let packet_id = PacketId::from(bytes[0]);
        log::debug!("packet_id = {}", packet_id);

        let payload_size = u32::from_be_bytes(bytes[1..5].try_into().expect("Incorrect length")) as usize;
        log::debug!("payload_size = {}", payload_size);

        if bytes.len() != SIZE_HEADER + payload_size {
            return Err(ParseError::mismatched_packet_size(packet_id, bytes.len(), payload_size));
        }

        // ================================================
        // Parse payload
        // ================================================
        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&bytes[5..(5 + payload_size) as usize]);

        let mut packet = Packet::new(packet_id, None);
        match packet_id {
            PacketId::Heartbeat => {}
            PacketId::HeartbeatAck => {}
            PacketId::RequestSendReplica => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::SendReplica => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::SendReplicaAck => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::AskIp => {}
            PacketId::AskIpAck => match payload_size {
                0 => {
                    return Err(ParseError::unavailable_master_ip());
                }
                4 => {
                    packet.ip_master = Some(Ipv4Addr::new(payload[0], payload[1], payload[2], payload[3]));
                }
                _ => {
                    return Err(ParseError::incorrect_payload_size_ask_ip_ack(payload.len()));
                }
            },
            PacketId::RequestFromClient => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::ResponseNodeIp => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::ClientUpload => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::DataNodeSendData => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::ClientRequestAck => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::StateSync => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::StateSyncAck => {
                // TODO: HoangLe [May-02]: Implement this
            }
            PacketId::Notify => {
                // TODO: HoangLe [May-02]: Implement this
            }
            _ => return Err(ParseError::incorrect_packet_id(packet_id as u8)),
        }

        packet.addr_sender = Some(stream.peer_addr().unwrap());

        log::debug!("{}", packet);

        return Ok(packet);
    }

    pub fn create_heartbeat() -> Packet {
        Packet {
            packet_id: PacketId::Heartbeat,
            ..Default::default()
        }
    }
    pub fn create_heartbeat_ack() -> Packet {
        Packet {
            packet_id: PacketId::HeartbeatAck,
            ..Default::default()
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
    pub fn create_ask_ip() -> Packet {
        Packet::new(PacketId::AskIp, None)
    }
    pub fn create_ask_ip_ack(addr_master: Option<&Ipv4Addr>) -> Packet {
        let mut packet = Packet::new(PacketId::AskIpAck, None);
        match addr_master {
            Some(addr_master) => {
                let payload = addr_master.octets().to_vec();
                packet.payload = Some(payload);
            }
            None => {}
        }

        packet
    }
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

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr_sender = match self.addr_sender {
            Some(addr_sender) => format!("{}", addr_sender),
            None => String::from("None"),
        };
        write!(f, "Packet: packet_id: {}, addr_sender: {}", self.packet_id, addr_sender)
    }
}
