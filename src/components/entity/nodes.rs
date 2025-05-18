use log;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use crate::components::{
    configs::Configs,
    // db::{FileInfoDB, NodeInfoDB},
    entity::node_roles::Role,
    errors::{NodeCreationError, NodeCreationErrorCode},
    packets::{Packet, PacketId},
};

// ================================================
// Definition
// ================================================

pub struct Node {
    configs: Configs,
    role: Role,

    // Use for communicating among threads inside node
    pub sender_receiver2processor: Option<Sender<Packet>>,
    pub receiver_receiver2processor: Option<Receiver<Packet>>,
    pub sender_processor2sender: Option<Sender<Packet>>,
    pub receiver_processor2sender: Option<Receiver<Packet>>,
    // For data management
    // TODO: HoangLe [May-12]: Store node and file info into these db
    // node_info: NodeInfoDB,
    // data_info: FileInfoDB,
}

// ================================================
// Implementation
// ================================================

impl Node {
    /// Create new node
    pub fn new(configs: Configs, role: Role) -> Node {
        let (sender_receiver2processor, receiver_receiver2processor) = channel::<Packet>();
        let (sender_processor2sender, receiver_processor2sender) = channel::<Packet>();

        Node {
            configs,
            role,
            sender_receiver2processor: Some(sender_receiver2processor),
            receiver_receiver2processor: Some(receiver_receiver2processor),
            sender_processor2sender: Some(sender_processor2sender),
            receiver_processor2sender: Some(receiver_processor2sender),
            // node_info: NodeInfoDB::intialize("node_info"),
            // data_info: FileInfoDB::intialize("file_info"),
        }
    }

    pub fn start(&mut self) {
        // ================================================
        // Declare different threads for different functions
        // ================================================
        let thread_receiver =
            match self.create_thread_receiver(SocketAddr::from(([127, 0, 0, 1], self.configs.env_port_receiver))) {
                Ok(handle) => handle,
                Err(err) => {
                    log::error!("{}", err);
                    return;
                }
            };
        let thread_processor = match self.create_thread_processor() {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };
        let thread_sender = match self.create_thread_sender() {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };

        // ================================================
        // Join threads
        // ================================================

        if let Err(err) = thread_receiver.join() {
            log::error!("Error as creating thread_receiver: {:?}", err);
            return;
        }
        if let Err(err) = thread_processor.join() {
            log::error!("Error as creating thread_processor: {:?}", err);
            return;
        }
        if let Err(err) = thread_sender.join() {
            log::error!("Error as creating thread_sender: {:?}", err);
            return;
        }
    }

    /// Create a thread dedicated for receiving incoming message
    fn create_thread_receiver(&mut self, addr: SocketAddr) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Receiver");

        let sender_receiver2processor = self.sender_receiver2processor.take().unwrap();

        Ok(thread::spawn(move || {
            let listener = match TcpListener::bind(&addr) {
                Ok(listener) => listener,
                Err(_) => {
                    log::error!("Cannot bind to {}", addr);
                    panic!();
                }
            };
            log::info!("Server starts at {}", addr);

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let packet = match Packet::from_stream(&mut stream) {
                            Ok(packet) => packet,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        // Send to thread Processor
                        if let Err(err) = sender_receiver2processor.send(packet) {
                            log::error!(
                                "Error as sending packet from thread:Receiver -> thread:Processor: err = {}",
                                err
                            );
                        };
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                }
            }
        }))
    }

    /// Create a thread dedicated for processing message
    fn create_thread_processor(&mut self) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Processor");

        let receiver_receiver2processor = self.receiver_receiver2processor.take().unwrap();
        let sender_processor2sender = self.sender_processor2sender.as_ref().unwrap().clone();
        let env_port_receiver = self.configs.env_port_receiver;
        let addr_dns = SocketAddr::new(IpAddr::V4(self.configs.env_ip_dns), self.configs.env_port_dns);
        let role = self.role;

        // ================================================
        // Execute 1st step of Initial procedure based on node's role
        // ================================================

        match self.role {
            Role::Master => {
                // Send its IP to DNS
                if let Err(err) = self
                    .sender_processor2sender
                    .as_ref()
                    .unwrap()
                    .send(Packet::create_notify(addr_dns.clone(), &self.role))
                {
                    log::error!("Error as sending Notify: {}", err);
                    return Err(NodeCreationError {
                        error_code: NodeCreationErrorCode::ProcessorThreadErr,
                    });
                }
            }
            Role::Data => {
                // Ask Master IP from DNS and notify to current master

                if let Err(err) = self
                    .sender_processor2sender
                    .as_ref()
                    .unwrap()
                    .send(Packet::create_ask_ip(addr_dns))
                {
                    log::error!("Error as sending AskIP: {}", err);
                    return Err(NodeCreationError {
                        error_code: NodeCreationErrorCode::ProcessorThreadErr,
                    });
                }
            }
            _ => {
                log::error!(
                    "Only Role::Master and Role::Data allow. This node has invalid role: {}",
                    self.role
                );
            }
        }

        // ================================================
        // Start processing loop
        // ================================================
        Ok(thread::spawn(move || {
            for packet in receiver_receiver2processor {
                log::debug!("Received: {}", packet);

                match packet.packet_id {
                    PacketId::Heartbeat => {
                        // TODO: HoangLe [May-03]: Send Heartbeat ACK
                    }
                    PacketId::HeartbeatAck => {
                        // TODO: HoangLe [May-03]: Record the healthy status
                    }
                    PacketId::AskIpAck => {
                        let payload = packet.payload.as_ref().unwrap();
                        let addr_master = SocketAddr::new(
                            IpAddr::V4(Ipv4Addr::new(payload[0], payload[1], payload[2], payload[3])),
                            env_port_receiver,
                        );

                        log::info!("Addr master: {}", addr_master);

                        if let Err(err) = sender_processor2sender.send(Packet::create_notify(addr_master, &role)) {
                            log::error!("Err as sending from thread:Processor -> thread:Sender: {}", err);
                            continue;
                        };
                    }
                    _ => {
                        log::error!("Unsupported packet type: {}", packet);
                        continue;
                    }
                }
            }
        }))
    }

    /// Create thread for sending packet
    fn create_thread_sender(&mut self) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Sender");

        let receiver_processor2sender = self.receiver_processor2sender.take().unwrap();

        Ok(thread::spawn(move || {
            for packet in receiver_processor2sender {
                let addr_receiver = match packet.addr_receiver {
                    Some(addr) => addr,
                    None => {
                        log::error!("Field 'addr_receiver' not specified.");
                        continue;
                    }
                };

                // Connect and send
                let mut stream = match TcpStream::connect(&addr_receiver) {
                    Ok(stream) => stream,
                    Err(_) => {
                        log::error!("Cannot connect to address: {}", addr_receiver);
                        continue;
                    }
                };

                let a = packet.to_bytes();
                if let Err(err) = stream.write_all(a.as_slice()) {
                    log::error!("Cannot send to address: {} : {}", &addr_receiver, err);
                }
            }
        }))
    }
}
