use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use std::str;

use log::{error, info};

// ================================================
// Definition
// ================================================
const BUFF_LEN: usize = 1024;

pub enum NodeType {
    Master,
    Data,
}

struct Address {
    ip: String,
    port: i32,
}

pub struct Node {
    node_type: NodeType,
    port_receiver: u32,
}

// ================================================
// Implementation
// ================================================

fn handle_connection(stream: &mut TcpStream) {
    let mut payload = Vec::<u8>::new();
    let mut buff: [u8; BUFF_LEN] = [0; BUFF_LEN];
    loop {
        let n = stream.read(&mut buff).unwrap();
        if n == 0 {
            break;
        } else {
            payload.extend_from_slice(&buff);

            if n < BUFF_LEN {
                break;
            }
        }
    }

    info!("No. bytes read in payload: {}", payload.len());
    let a = str::from_utf8(payload.as_slice()).unwrap();
    println!("{}", a);
    // for byte in payload {
    //     println!("{}", str::from_utf8())
    // }
}

impl Address {
    pub fn to_str(self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl Node {
    /// Create new node
    pub fn new(node_type: NodeType, port_receiver: u32) -> Node {
        Node {
            node_type,
            port_receiver,
        }
    }

    pub fn initialize(node_type: NodeType, port_receiver: u32) -> Node {
        Node {
            node_type,
            port_receiver,
        }
    }

    /// Create a thread dedicated for receiving incoming message
    pub fn create_thread_receiver(self) {
        // TODO: HoangLe [Apr-13]: Implement this

        let addr_struct = Address {
            ip: String::from("127.0.0.1"),
            port: 7888,
        };
        let addr = addr_struct.to_str();

        let listener = match TcpListener::bind(&addr) {
            Ok(listener) => listener,
            Err(_) => {
                error!("Cannot bind to {}", addr);
                panic!();
            }
        };
        info!("Server starts at {}", addr);

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            info!("Connection established");

            handle_connection(&mut stream);
        }
    }

    /// Create a thread dedicated for processing message
    pub fn create_thread_processor() {}

    pub fn connect() {
        // TODO: HoangLe [Apr-13]: Implement this
    }
}
