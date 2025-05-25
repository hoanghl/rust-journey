use log;

use std::{
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use crate::components::{
    configs::Configs,
    db::NodeInfoDB,
    entity::node_roles::Role,
    errors::NodeCreationError,
    packets::{Packet, PacketId},
};

// ================================================
// Definition
// ================================================

pub struct Node {
    configs: Configs,
    role: Role,
}

// ================================================
// Implementation
// ================================================

impl Node {
    /// Create new node
    pub fn new(configs: Configs, role: Role) -> Node {
        Node { configs, role }
    }

    pub fn start(&mut self) {
        // Use for communicating among threads inside node
        let (sender_receiver2processor, receiver_receiver2processor) = channel::<Packet>();
        let (sender_processor2sender, receiver_processor2sender) = channel::<Packet>();
        // ================================================
        // Declare different threads for different functions
        // ================================================

        let thread_receiver = match self.create_thread_receiver(sender_receiver2processor) {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };
        let thread_sender = match self.create_thread_sender(receiver_processor2sender) {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };

        // ================================================
        // Start processing packets
        // ================================================
        self.trigger_processor(&receiver_receiver2processor, &sender_processor2sender);

        // ================================================
        // Join threads
        // ================================================

        if let Err(err) = thread_receiver.join() {
            log::error!("Error as creating thread_receiver: {:?}", err);
            return;
        }
        if let Err(err) = thread_sender.join() {
            log::error!("Error as creating thread_sender: {:?}", err);
            return;
        }
    }

    /// Create a thread dedicated for receiving incoming message
    fn create_thread_receiver(
        &mut self,
        sender_receiver2processor: Sender<Packet>,
    ) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Receiver");

        let port = match self.role {
            Role::DNS => self.configs.env_port_dns,
            _ => self.configs.env_port_receiver,
        };
        let addr_node = SocketAddr::from(([127, 0, 0, 1], port));
        Ok(thread::spawn(move || {
            let listener = match TcpListener::bind(&addr_node) {
                Ok(listener) => listener,
                Err(_) => {
                    log::error!("Cannot bind to {}", addr_node);
                    panic!();
                }
            };
            log::info!("Server starts at {}", addr_node);

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

    /// Create thread for sending packet
    fn create_thread_sender(
        &mut self,
        receiver_processor2sender: Receiver<Packet>,
    ) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Sender");

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

    /// Gracefully shutdown thread:Receiver and thread:Sender
    fn trigger_graceful_shutdown(&self) {
        // TODO: HoangLe [May-18]: Implement this
    }

    /// Start processor
    fn trigger_processor(
        &mut self,
        receiver_receiver2processor: &Receiver<Packet>,
        sender_processor2sender: &Sender<Packet>,
    ) {
        let addr_dns: SocketAddr = SocketAddr::new(IpAddr::V4(self.configs.env_ip_dns), self.configs.env_port_dns);
        let mut addr_master: Option<SocketAddr> = None;
        let addr_current = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            self.configs.env_port_receiver,
        ));

        // For data management
        let node_info = NodeInfoDB::intialize("node_info");
        // let data_info = FileInfoDB::intialize(db_name)

        // For counter
        let mut last_ts: Option<SystemTime> = None;

        // ================================================
        // Execute 1st step of Initial procedure based on node's role
        // ================================================

        match self.role {
            Role::Master => {
                // Send its IP to DNS
                if let Err(err) = sender_processor2sender.send(Packet::create_notify(
                    addr_dns.clone(),
                    &self.role,
                    addr_current.clone(),
                )) {
                    log::error!("Error as sending Notify: {}", err);
                    self.trigger_graceful_shutdown();
                }
            }
            Role::Data => {
                // Ask Master IP from DNS and notify to current master

                if let Err(err) =
                    sender_processor2sender.send(Packet::create_ask_ip(addr_dns, Some(self.configs.env_port_receiver)))
                {
                    log::error!("Error as sending AskIP: {}", err);
                    self.trigger_graceful_shutdown();
                }
            }
            _ => {}
        }

        // ================================================
        // Start processing loop
        // ================================================
        loop {
            if let Ok(packet) =
                receiver_receiver2processor.recv_timeout(Duration::from_secs(self.configs.timeout_channel_wait))
            {
                log::debug!("Received: {}", packet);

                let addr_sender = match packet.addr_sender {
                    None => {
                        log::error!("Attribute 'addr_sender' in packet not existed.");
                        continue;
                    }
                    Some(addr) => addr,
                };

                match self.role {
                    Role::Default => {
                        log::error!("Role:Default not allow. Exitting.");
                        break;
                    }
                    Role::DNS => {
                        match packet.packet_id {
                            PacketId::AskIp => {
                                // Data/Client --AskIp-> DNS
                                match addr_master {
                                    None => {
                                        _forward_packet(
                                            sender_processor2sender,
                                            Packet::create_ask_ip_ack(addr_sender, None),
                                        );
                                    }
                                    Some(addr_master) => {
                                        _forward_packet(
                                            sender_processor2sender,
                                            Packet::create_ask_ip_ack(addr_sender, Some(&addr_master)),
                                        );
                                    }
                                };
                            }
                            PacketId::Notify => {
                                // Master --Notify-> DNS

                                addr_master = packet.addr_sender;
                                log::info!("Address Master just notified: {}", &addr_master.as_ref().unwrap());
                            }
                            _ => {
                                log::error!("Unsupported packet type: {}", packet);
                                continue;
                            }
                        };
                    }
                    Role::Master => {
                        match packet.packet_id {
                            PacketId::HeartbeatAck => {
                                if let Some(node_id) = packet.node_id {
                                    match SocketAddrV4::from_str(node_id.as_str()) {
                                        Ok(addr) => {
                                            if let Err(err) =
                                                node_info.upsert(addr.ip().clone(), addr.port(), Role::Data)
                                            {
                                                log::error!("Error as UPSERT: {}", err);
                                            }
                                        }
                                        Err(err) => {
                                            log::error!(
                                                "Cannot parse following node_id to SocketAddrV4: {} | Err: {}",
                                                node_id,
                                                err
                                            );
                                        }
                                    }
                                }
                            }
                            PacketId::Notify => {
                                log::info!("Master receives NOTIFY from: {:?}", packet.addr_sender);

                                match packet.addr_sender {
                                    Some(addr_sender) => {
                                        // data_nodes.push(addr_sender);

                                        if let IpAddr::V4(ip) = addr_sender.ip() {
                                            let _ = node_info.upsert(ip, addr_sender.port(), Role::Data);

                                            log::info!("Master added new Data node: {}", addr_sender);
                                        }
                                    }
                                    None => {
                                        log::error!("NOTIFY packet contains no sender' address");
                                    }
                                }
                            }
                            _ => {
                                log::error!("Unsupported packet type: {}", packet);
                                continue;
                            }
                        }
                    }
                    Role::Data => match packet.packet_id {
                        PacketId::Heartbeat => {
                            _forward_packet(
                                sender_processor2sender,
                                Packet::create_heartbeat_ack(addr_master.clone().unwrap(), addr_current.clone()),
                            );
                        }
                        PacketId::AskIpAck => match packet.addr_master {
                            None => {
                                log::error!("Received packet not contain address of Master");
                                continue;
                            }
                            Some(addr) => {
                                log::debug!("Addr master: {:?}", addr);

                                addr_master = Some(addr.clone());

                                _forward_packet(
                                    sender_processor2sender,
                                    Packet::create_notify(addr, &self.role, addr_current.clone()),
                                );
                            }
                        },
                        _ => {
                            log::error!("Unsupported packet type: {}", packet);
                            continue;
                        }
                    },
                }
            }

            // If current node is Master, check timer and send Heartbeat
            match self.role {
                Role::Master => {
                    if last_ts.is_none() {
                        last_ts = Some(SystemTime::now());
                    } else {
                        match SystemTime::now().duration_since(last_ts.unwrap()) {
                            Ok(n) => {
                                if n.as_secs() >= self.configs.interval_heartbeat {
                                    last_ts = Some(SystemTime::now());

                                    // Send heartbeat
                                    if let Ok(data_nodes) = node_info.get_data_nodes() {
                                        for node in &data_nodes {
                                            match node.ip {
                                                None => {
                                                    log::error!(
                                                        "Cannot retrieve ip from node with node_id = {}",
                                                        node.node_id
                                                    );
                                                    continue;
                                                }
                                                Some(ip) => {
                                                    let addr = SocketAddr::V4(SocketAddrV4::new(ip, node.port));

                                                    log::info!("Send HEARTBEAT to {}", addr);
                                                    _forward_packet(
                                                        sender_processor2sender,
                                                        Packet::create_heartbeat(addr),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                log::error!("{}", err);
                            }
                        }
                    }
                }
                _ => {
                    last_ts = None;
                }
            }
        }
    }
}

fn _forward_packet(sender_processor2sender: &Sender<Packet>, packet: Packet) {
    if let Err(err) = sender_processor2sender.send(packet) {
        log::error!("Err as sending from thread:Processor -> thread:Sender: {}", err);
    };
}
