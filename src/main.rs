use std::{
    env,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};

use dotenv::dotenv;

mod node;
mod packets;

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

    // // ================================================
    // // Start server
    // // ================================================
    // let node_data = node::Node::initialize(node::NodeType::Data, port_receiver);

    // node_data.create_thread_receiver();

    // let listerner = TcpListener::bind("127.0.0.1:7878").unwrap();
    // for stream in listerner.incoming() {
    //     let stream = stream.unwrap();
    //     thread::spawn(|| {
    //         handle_connection(stream);
    //     });
    // }

    let mut list = Vec::from(&[1, 2, 3]);
    println!("before defining closure: {list:?}");

    let mut only_borrows = || {
        list.push(999);
        print!("Inside closure: {list:?}");
    };

    // println!("Before calling closure: {list:?}");
    only_borrows();
    println!("After calling closure: {list:?}");
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request: Vec<_> = buf_reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", request);
}
