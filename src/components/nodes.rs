use log;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::components::packets::{self, Packet};

// ================================================
// Definition
// ================================================
const BUFF_LEN: usize = 1024;

pub struct Node {
    port_receiver: u32,

    sender_receiver2processor: Option<Sender<Packet>>,
    receiver_receiver2processor: Option<Receiver<Packet>>,
    sender_processor2sender: Option<Sender<Packet>>,
    receiver_processor2sender: Option<Receiver<Packet>>,
}

// struct Worker<T> {
//     sender_main2worker: Option<Sender<T>>,
//     rcv_main2worker: Option<Receiver<T>>,
//     sender_worker2main: Option<Sender<T>>,
//     rcv_worker2main: Option<Receiver<T>>,
//     worker: Option<JoinHandle<()>>,
// }

// ================================================
// Implementation
// ================================================

// fn handle_connection(stream: &mut TcpStream) {
//     let mut bytes = Vec::<u8>::new();
//     let mut buff: [u8; BUFF_LEN] = [0; BUFF_LEN];
//     loop {
//         let n = stream.read(&mut buff).unwrap();
//         if n == 0 {
//             break;
//         } else {
//             bytes.extend_from_slice(&buff[0..n]);

//             if n < BUFF_LEN {
//                 break;
//             }
//         }
//     }

//     log::debug!("No. bytes read in payload: {}", bytes.len());

//     let packet = packets::Packet::from_bytes(bytes.as_ref());

//     log::debug!("Parse packet: {}", packet.packet_id);
// }

// impl Worker<Packet> {
//     pub fn new() -> Worker<Packet> {
//         let (sender_main2worker, rcv_main2worker) = channel::<Packet>();
//         let (sender_worker2main, rcv_worker2main) = channel::<Packet>();

//         Worker {
//             sender_main2worker: Some(sender_main2worker),
//             rcv_main2worker: Some(rcv_main2worker),
//             sender_worker2main: Some(sender_worker2main),
//             rcv_worker2main: Some(rcv_worker2main),
//             worker: None,
//         }
//     }

//     pub fn start(&mut self, f: fn(Sender<Packet>, Receiver<Packet>)) {
//         let sender_worker2main = self.sender_worker2main.take().unwrap();
//         let rcv_main2worker = self.rcv_main2worker.take().unwrap();
//         self.worker = Some(thread::spawn(move || f(sender_worker2main, rcv_main2worker)));
//     }

//     pub fn get(&self) {
//         for received in self.rcv_worker2main.as_ref().unwrap() {
//             println!("{}", received.packet_id);
//         }
//     }
// }

impl Node {
    /// Create new node
    pub fn new(port_receiver: u32) -> Node {
        let (sender_receiver2processor, receiver_receiver2processor) = channel::<Packet>();
        let (sender_processor2sender, receiver_processor2sender) = channel::<Packet>();

        Node {
            port_receiver,
            sender_receiver2processor: Some(sender_receiver2processor),
            receiver_receiver2processor: Some(receiver_receiver2processor),
            sender_processor2sender: Some(sender_processor2sender),
            receiver_processor2sender: Some(receiver_processor2sender),
        }
    }

    pub fn start(&mut self) {
        let thread1 = self.create_thread_receiver(SocketAddr::from(([127, 0, 0, 1], self.port_receiver as u16)));
        let thread2 = self.create_thread_processor();
        let thread3 = self.create_thread_sender();

        thread1.join();
        thread2.join();
        thread3.join();
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
                let mut stream = stream.unwrap();

                log::info!("Receive incomming connection from: {}", stream.peer_addr().unwrap());

                // Parse
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

                log::debug!("No. bytes read in payload: {}", bytes.len());

                let packet = match packets::Packet::from_bytes(bytes.as_ref()) {
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
        })
    }

    /// Create a thread dedicated for processing message
    fn create_thread_processor(&mut self) -> JoinHandle<()> {
        log::info!("Creating thread: Processor");

        let receiver_receiver2processor = self.receiver_receiver2processor.take().unwrap();
        let sender_processor2sender = self.sender_processor2sender.take().unwrap();

        thread::spawn(move || {
            for received in receiver_receiver2processor.recv() {
                // TODO: HoangLe [Apr-28]: Process data
                log::info!("Received packet: {}", received.packet_id);
            }
        })
    }

    /// Create thread for sending packet
    fn create_thread_sender(&mut self) -> JoinHandle<()> {
        log::info!("Creating thread: Sender");

        let receiver_processor2sender = self.receiver_processor2sender.take().unwrap();

        thread::spawn(move || {
            for packet in receiver_processor2sender.recv() {
                let addr_receiver = match packet.addr_receiver {
                    Some(addr) => addr.to_str(),
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

    // fn handler_receiver() {
    //     loop {
    //         let packet = Packet::new(packets::PacketId::Heartbeat, None);

    //         if let Err(_) = sender_worker2main.send(packet) {
    //             panic!();
    //         };
    //         // thread::sleep(Duration::from_millis(5));
    //     }
    // }

    // C

    // pub fn connect() {
    //     // TODO: HoangLe [Apr-13]: Implement this
    // }
}
