mod components;

use std::env;

use components::*;
use dotenv::dotenv;

fn main() {
    // ================å===============================
    // Intial configurations
    // ================================================
    dotenv().ok();

    // Set up logger
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let port_receiver = match env::var("PORT_RECEIVER") {
        Ok(value) => value.parse::<u32>().unwrap(),
        Err(_) => panic!("env 'PORT_RECEIVER' not existed"),
    };

    // Parse arguments
    let args: Vec<String> = env::args().collect();

    // ================================================
    // Start server
    // ================================================
    let addr_server = address::Address {
        ip: String::from("127.0.0.1"),
        port: port_receiver,
    }
    .to_str();

    let node = nodes::Node::new(port_receiver);
    match args[1].as_str() {
        "receiver" => {
            node.create_thread_receiver(addr_server);
        }
        "sender" => {
            node.create_thread_processor(addr_server);
        }
        _ => panic!("Incorrect argument: {}", args[1]),
    }
}

// fn handle_connection(stream: TcpStream) {
//     let mut buf_reader = BufReader::new(&stream);
//     let mut request = Vec::<u8>::new();
//     if let Err(_) = buf_reader.read_to_end(&mut request) {
//         error!("Error as reading bytes from {}", stream.peer_addr().unwrap());
//         return;
//     }

//     // let request: Vec<_> = buf_reader
//     //     .lines()
//     //     .map(|line| line.unwrap())
//     //     .take_while(|line| !line.is_empty())
//     //     .collect();

//     println!("Request: {:#?}", request);
// }
