use components::{
    configs::Configs,
    entity::{client::Client, node_roles::Role, nodes::Node},
};

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
        "master" => {
            let mut node = Node::new(configs, Role::Master);
            node.start();
        }
        "data" => {
            let mut node = Node::new(configs, Role::Data);
            node.start();
        }
        "dns" => {
            let mut node = Node::new(configs, Role::DNS);
            node.start()
        }
        "client" => {
            let client = Client::new(&configs);
            client.ask_master_ip();
        }
        _ => panic!("First argument must be a valid mode"),
    };
}
