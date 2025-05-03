use dotenv::dotenv;
use std::net::Ipv4Addr;

use std::env;
use std::str::FromStr;

pub struct Configs {
    pub env_ip_dns: Ipv4Addr,
    pub env_port_receiver: u16,
    pub env_port_dns: u16,

    pub args: Vec<String>,
}

impl Configs {
    pub fn initialize() -> Configs {
        // Read .env file and parse
        dotenv().ok();

        let ip_dns = match env::var("IP_DNS") {
            Ok(value) => Ipv4Addr::from_str(value.parse::<String>().unwrap().as_str())
                .expect("Cannot parse env 'IP_DNS' to correct IP address format"),
            Err(_) => panic!("env 'IP_DNS' not existed"),
        };
        let port_receiver = match env::var("PORT_RECEIVER") {
            Ok(value) => value.parse::<u16>().unwrap(),
            Err(_) => panic!("env 'PORT_RECEIVER' not existed"),
        };
        let port_dns = match env::var("PORT_DNS") {
            Ok(value) => value.parse::<u16>().unwrap(),
            Err(_) => panic!("env 'PORT_DNS' not existed"),
        };

        // Parse arguments
        // TODO: HoangLe [May-02]: Enhance arg parsing
        let args: Vec<String> = env::args().collect();

        // Set up logger
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "info");
        }
        env_logger::init();

        Configs {
            env_ip_dns: ip_dns,
            env_port_receiver: port_receiver,
            env_port_dns: port_dns,
            args,
        }
    }
}
