// TODO: HoangLe [May-02]: Implement this

use std::{
    io::Write,
    net::{IpAddr, SocketAddr, TcpStream},
};

use crate::components::{configs::Configs, packets::Packet};

// ================================================
// Definitions
// ================================================
pub struct Client {
    addr_dns: SocketAddr,
}

// ================================================
// Implementations
// ================================================
impl Client {
    pub fn new(configs: &Configs) -> Client {
        Client {
            addr_dns: SocketAddr::new(IpAddr::V4(configs.env_ip_dns), configs.env_port_dns),
        }
    }

    pub fn ask_master_ip(&self) {
        match TcpStream::connect(self.addr_dns) {
            Ok(mut stream) => {
                let _ = stream.write_all(Packet::create_ask_ip().to_bytes().as_slice());
                match Packet::from_stream(&mut stream) {
                    Ok(packet_reply) => match packet_reply.ip_master {
                        Some(ip_master) => {
                            log::info!("Master has address: {}", ip_master);
                        }
                        None => {
                            log::info!("Address for current Master not available");
                        }
                    },
                    Err(error) => {
                        log::error!("{}", error);
                    }
                }
            }
            Err(e) => {
                log::error!("Cannot connect to: {}: {}", self.addr_dns, e);
            }
        }
    }
}
