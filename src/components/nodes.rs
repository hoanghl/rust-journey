use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use log;

use crate::components::packets;

// ================================================
// Definition
// ================================================
const BUFF_LEN: usize = 1024;

pub struct Node {
    port_receiver: u32,
}

// ================================================
// Implementation
// ================================================

fn handle_connection(stream: &mut TcpStream) {
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

    let packet = packets::Packet::from_bytes(bytes.as_ref());

    log::debug!("Parse packet: {}", packet.packet_id);
}

impl Node {
    /// Create new node
    pub fn new(port_receiver: u32) -> Node {
        Node { port_receiver }
    }

    /// Create a thread dedicated for receiving incoming message
    pub fn create_thread_receiver(&self, addr: String) {
        // TODO: HoangLe [Apr-13]: Implement this

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

            handle_connection(&mut stream);
        }
    }

    /// Create a thread dedicated for processing message
    pub fn create_thread_processor(&self, addr: String) {
        // TODO: HoangLe [Apr-21]: Continue implementing this

        match TcpStream::connect(&addr) {
            Ok(mut stream) => {
                let packet = packets::Packet::new(packets::PacketId::Heartbeat, None);
                let a = packet.to_bytes();
                if let Err(e) = stream.write_all(a.as_slice()) {
                    panic!("Cannot send to address: {} : {}", &addr, e);
                }
            }
            Err(_) => {
                panic!("Cannot connect to address: {}", addr)
            }
        }
    }

    // C

    // pub fn connect() {
    //     // TODO: HoangLe [Apr-13]: Implement this
    // }
}
