use components::nodes::Node;
use dotenv::dotenv;
// use log;

use std::{
    env,
    // io::{BufRead, BufReader, Read},
    // net::{TcpListener, TcpStream},
};

mod components;

fn main() {
    // ================================================
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
    // Establish server
    // ================================================
    Node::new(port_receiver).start();

    // let ip = "127.0.0.1";
    // let port = 7878;
    // let addr = format!("{}:{}", ip, port);
    // let listener = match TcpListener::bind(&addr) {
    //     Ok(listener) => listener,
    //     Err(e) => {
    //         log::error!("Cannot bind to address: {}: {}", &addr, e);
    //         panic!();
    //     }
    // };

    // for result_listenner in listener.incoming() {
    //     let stream = match result_listenner {
    //         Ok(stream) => stream,
    //         Err(e) => panic!("{}", e),
    //     };

    //     let a = thread::spawn(move || {
    //
    //         handle_connection(stream);
    //     });
    // }
}

// fn handle_connection(mut stream: TcpStream) {
//     log::info!("Connection established with: {}", stream.peer_addr().unwrap());

// let mut request = String::new();

// println!("Request: {}", request);

// stream.write(&[0]);
// }
