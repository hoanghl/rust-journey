use std::net::Ipv4Addr;

use components::{client::Client, configs::Configs, dns::DNS, nodes::Node};

mod components;

fn main() {
    // ================================================
    // Intialize configs
    // ================================================
    let configs = Configs::initialize();

    // ================================================
    // Establish server
    // ================================================
    match configs.args[1].as_str() {
        "master" | "data" => {
            Node::new(&configs).start();
        }
        "dns" => {
            let mut dns = DNS::new(&configs);

            // FIXME: HoangLe [May-03]: Remove the following after testing
            dns.set_addr_master(Ipv4Addr::new(123, 111, 22, 33));
            dns.start()
        }
        "client" => {
            let client = Client::new(&configs);
            client.ask_master_ip();
        }
        _ => panic!("First argument must be a valid mode"),
    }
}
