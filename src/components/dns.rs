// TODO: HoangLe [May-02]: Implement this

use crate::components::{
    configs::Configs,
    packets::{Packet, PacketId},
};
use log;
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, TcpListener},
};

// ================================================
// Definitions
// ================================================
pub struct DNS<'a> {
    configs: &'a Configs,

    addr_master: Option<Ipv4Addr>,
}

// ================================================
// Implementations
// ================================================
impl<'a> DNS<'a> {
    pub fn new(configs: &Configs) -> DNS {
        DNS {
            addr_master: None,
            configs,
        }
    }

    pub fn set_addr_master(&mut self, addr_master: Ipv4Addr) {
        self.addr_master = Some(addr_master);
    }

    pub fn get_addr_master(&self) -> Option<&Ipv4Addr> {
        if self.addr_master == None {
            return None;
        }
        return self.addr_master.as_ref();
    }

    pub fn start(self) {
        let addr = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            self.configs.env_port_dns,
        );
        log::info!("Start DNS server at: {}", &addr);

        let listener = match TcpListener::bind(&addr) {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Cannot bind to address: {}: {}", addr, e);
            }
        };

        for result in listener.incoming() {
            let mut stream = match result {
                Ok(stream) => stream,
                Err(e) => {
                    log::error!("Error as getting from listener: {}", e);
                    continue;
                }
            };

            // Parse stream
            let packet = match Packet::from_stream(&mut stream) {
                Ok(packet) => packet,
                Err(e) => {
                    log::error!("{}", e);
                    continue;
                }
            };

            // Process request
            match packet.packet_id {
                PacketId::AskIp => {
                    let packet_reply = Packet::create_ask_ip_ack(self.get_addr_master());

                    let _ = stream.write_all(packet_reply.to_bytes().as_slice());
                }
                _ => {
                    log::error!(
                        "Invalid packet type from: {} - packet_type: {}",
                        packet
                            .addr_sender
                            .expect("Address of sender not specified as forming the packet"),
                        packet.packet_id
                    );
                }
            }
        }
    }
}
