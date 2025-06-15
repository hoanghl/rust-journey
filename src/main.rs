use components::{
    configs::Configs,
    entity::{client::Client, node_roles::Role, nodes::Node},
    packets::Action,
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
    match configs.args.role {
        Role::Master => {
            let mut node = Node::new(configs, Role::Master);
            node.start();
        }
        Role::Data => {
            let mut node = Node::new(configs, Role::Data);
            node.start();
        }
        Role::DNS => {
            let mut node = Node::new(configs, Role::DNS);
            node.start()
        }
        Role::Client => {
            let mut client = Client::new(&configs);
            client.start(Action::Write);
        }
        _ => panic!("Invalid role argument"),
    };
}
