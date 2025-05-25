use dotenv::dotenv;
use std::net::Ipv4Addr;

use std::env;
use std::str::FromStr;

pub struct Configs {
    pub env_ip_dns: Ipv4Addr,
    pub env_port_receiver: u16,
    pub env_port_dns: u16,
    pub interval_heartbeat: u64,
    pub timeout_channel_wait: u64,

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
        let mut port_receiver = match env::var("PORT_RECEIVER") {
            Ok(value) => value.parse::<u16>().unwrap(),
            Err(_) => panic!("env 'PORT_RECEIVER' not existed"),
        };
        let port_dns = match env::var("PORT_DNS") {
            Ok(value) => value.parse::<u16>().unwrap(),
            Err(_) => panic!("env 'PORT_DNS' not existed"),
        };
        let interval_heartbeat = match env::var("HEARTBEAT_INTERVAL_SECOND") {
            Ok(value) => value.parse::<u64>().unwrap(),
            Err(_) => panic!("env 'HEARTBEAT_INTERVAL_SECOND' not existed"),
        };
        let timeout_channel_wait = match env::var("TIMEOUT_CHANNEL_WAIT") {
            Ok(value) => value.parse::<u64>().unwrap(),
            Err(_) => 1,
        };

        // Parse arguments
        // TODO: HoangLe [May-02]: Enhance arg parsing
        let args: Vec<String> = env::args().collect();

        // Set up logger
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "info");
        }
        env_logger::init();

        // Override some config
        if args.len() >= 3 {
            match args[2].parse::<u16>() {
                Ok(port) => {
                    log::info!("'port' argument specified. Override the default value.");
                    port_receiver = port;
                }
                Err(_) => {
                    log::error!("2nd argument specified but not valid port value: {}", args[2]);
                }
            }
        }

        Configs {
            env_ip_dns: ip_dns,
            env_port_receiver: port_receiver,
            env_port_dns: port_dns,
            interval_heartbeat,
            timeout_channel_wait,
            args,
        }
    }
}
