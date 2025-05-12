use log;

use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use crate::components::{
    configs::Configs,
    db::{FileInfoDB, NodeInfoDB},
    packets::{self, Packet, PacketId},
};

// ================================================
// Definition
// ================================================

pub struct Node<'a> {
    configs: &'a Configs,

    // Use for communicating among threads inside node
    sender_receiver2processor: Option<Sender<Packet>>,
    receiver_receiver2processor: Option<Receiver<Packet>>,
    sender_processor2sender: Option<Sender<Packet>>,
    receiver_processor2sender: Option<Receiver<Packet>>,

    // For data management
    // TODO: HoangLe [May-12]: Store node and file info into these db
    node_info: NodeInfoDB,
    data_info: FileInfoDB,
}

// ================================================
// Implementation
// ================================================

impl<'a> Node<'a> {
    /// Create new node
    pub fn new(configs: &Configs) -> Node {
        let (sender_receiver2processor, receiver_receiver2processor) = channel::<Packet>();
        let (sender_processor2sender, receiver_processor2sender) = channel::<Packet>();

        Node {
            configs,
            sender_receiver2processor: Some(sender_receiver2processor),
            receiver_receiver2processor: Some(receiver_receiver2processor),
            sender_processor2sender: Some(sender_processor2sender),
            receiver_processor2sender: Some(receiver_processor2sender),
            node_info: NodeInfoDB::intialize("node_info"),
            data_info: FileInfoDB::intialize("file_info"),
        }
    }

    pub fn start(&mut self) {
        let thread1 = self.create_thread_receiver(SocketAddr::from(([127, 0, 0, 1], self.configs.env_port_receiver)));
        let thread2 = self.create_thread_processor();
        let thread3 = self.create_thread_sender();

        let _ = thread1.join();
        let _ = thread2.join();
        let _ = thread3.join();
    }

    /// Create a thread dedicated for receiving incoming message
    fn create_thread_receiver(&mut self, addr: SocketAddr) -> JoinHandle<()> {
        log::info!("Creating thread: Receiver");

        // let mut worker_receiver = Worker::<packets::Packet>::new();
        // worker_receiver.start(Node::receiver);
        // worker_receiver.get();

        let sender_receiver2processor = self.sender_receiver2processor.take().unwrap();

        thread::spawn(move || {
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

                        // Send to another thread
                        // TODO: HoangLe [Apr-28]: Handle error from this
                        let _ = sender_receiver2processor.send(packet);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                }
            }
        })
    }

    /// Create a thread dedicated for processing message
    fn create_thread_processor(&mut self) -> JoinHandle<()> {
        log::info!("Creating thread: Processor");

        let receiver_receiver2processor = self.receiver_receiver2processor.take().unwrap();
        let sender_processor2sender = self.sender_processor2sender.take().unwrap().clone();

        thread::spawn(move || {
            for packet in receiver_receiver2processor {
                log::debug!("Received: {}", packet);

                match packet.packet_id {
                    PacketId::Heartbeat => {
                        // TODO: HoangLe [May-03]: Send Heartbeat ACK
                    }
                    PacketId::HeartbeatAck => {
                        // TODO: HoangLe [May-03]: Record the healthy status
                    }
                    _ => {
                        log::error!("Unsupported packet type: {}", packet);
                        continue;
                    }
                }
            }
        })
    }

    /// Create thread for sending packet
    fn create_thread_sender(&mut self) -> JoinHandle<()> {
        log::info!("Creating thread: Sender");

        let receiver_processor2sender = self.receiver_processor2sender.take().unwrap();

        thread::spawn(move || {
            for packet in receiver_processor2sender {
                let addr_receiver = match packet.addr_receiver {
                    Some(addr) => addr,
                    None => {
                        log::error!("Field 'addr_receiver' not specified.");
                        continue;
                    }
                };

                match TcpStream::connect(&addr_receiver) {
                    Ok(mut stream) => {
                        let packet = packets::Packet::new(packets::PacketId::Heartbeat, None);
                        let a = packet.to_bytes();
                        if let Err(e) = stream.write_all(a.as_slice()) {
                            log::error!("Cannot send to address: {} : {}", &addr_receiver, e);
                        }
                    }
                    Err(_) => {
                        log::error!("Cannot connect to address: {}", addr_receiver)
                    }
                }
            }
        })
    }

    // pub fn connect() {
    //     // TODO: HoangLe [Apr-13]: Implement this
    // }
}
