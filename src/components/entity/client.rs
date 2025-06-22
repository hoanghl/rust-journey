use std::{
    fs::File,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use log;

use crate::components::{
    configs::Configs,
    errors::NodeCreationError,
    packets::{forward_packet, wait_packet, Action, Packet, PacketId},
};

// ================================================
// Definitions
// ================================================
pub struct Client<'conf> {
    configs: &'conf Configs,
}

// ================================================
// Implementations
// ================================================
impl<'conf> Client<'conf> {
    pub fn new(configs: &Configs) -> Client {
        Client { configs }
    }

    pub fn start(&mut self, action: Action) {
        // Use for communicating among threads inside node
        let (sender_receiver2processor, receiver_receiver2processor) = channel::<Packet>();
        let (sender_processor2sender, receiver_processor2sender) = channel::<Packet>();
        // ================================================
        // Declare different threads for different functions
        // ================================================
        let flag_stop = Arc::new(AtomicBool::new(false));

        let thread_receiver = match self.create_thread_receiver(sender_receiver2processor, &flag_stop) {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };
        let thread_sender = match self.create_thread_sender(receiver_processor2sender, &flag_stop) {
            Ok(handle) => handle,
            Err(err) => {
                log::error!("{}", err);
                return;
            }
        };

        // ================================================
        // Start processing packets
        // ================================================
        match action {
            Action::Read => {
                // TODO: HoangLe [Jun-14]: Implement this
            }
            Action::Write => {
                self.send_file(&receiver_receiver2processor, &sender_processor2sender);
            }
        }

        // ================================================
        // Join threads
        // ================================================
        self.shutdown_gracefully(&flag_stop);

        if let Err(err) = thread_receiver.join() {
            log::error!("Error as creating thread_receiver: {:?}", err);
            return;
        }
        if let Err(err) = thread_sender.join() {
            log::error!("Error as creating thread_sender: {:?}", err);
            return;
        }
    }

    fn shutdown_gracefully(&self, flag_stop: &Arc<AtomicBool>) {
        flag_stop.store(true, Ordering::Relaxed);

        if let Err(err) = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], self.configs.args.port))) {
            log::error!("Error as executing gracefull shutdown: {}", err);
        };
    }

    /// Create a thread dedicated for receiving incoming message
    fn create_thread_receiver(
        &mut self,
        sender_receiver2processor: Sender<Packet>,
        flag_stop: &Arc<AtomicBool>,
    ) -> Result<JoinHandle<()>, NodeCreationError> {
        log::info!("Creating thread: Receiver");

        let port = self.configs.args.port;
        let addr_node = SocketAddr::from(([127, 0, 0, 1], port));
        let flag = Arc::clone(&flag_stop);

        Ok(thread::spawn(move || {
            let listener = match TcpListener::bind(&addr_node) {
                Ok(listener) => listener,
                Err(_) => {
                    log::error!("Cannot bind to {}", addr_node);
                    panic!();
                }
            };
            log::info!("Server starts at {}", addr_node);

            for stream in listener.incoming() {
                if flag.load(Ordering::Relaxed) {
                    break;
                }

                match stream {
                    Ok(mut stream) => {
                        let packet = match Packet::from_stream(&mut stream) {
                            Ok(packet) => packet,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        // Send to thread Processor
                        if let Err(err) = sender_receiver2processor.send(packet) {
                            log::error!(
                                "Error as sending packet from thread:Receiver -> thread:Processor: err = {}",
                                err
                            );
                        };
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                }
            }
        }))
    }

    /// Create thread for sending packet
    fn create_thread_sender(
        &mut self,
        receiver_processor2sender: Receiver<Packet>,
        flag_stop: &Arc<AtomicBool>,
    ) -> Result<JoinHandle<()>, NodeCreationError> {
        // NOTE: HoangLe [Jun-14]: Consider removing this implementation after converting Node to trait
        log::info!("Creating thread: Sender");

        let duration = self.configs.timeout_chan_wait;
        let flag = Arc::clone(&flag_stop);
        Ok(thread::spawn(move || {
            loop {
                if flag.load(Ordering::Relaxed) {
                    break;
                }

                let packet = match receiver_processor2sender.recv_timeout(Duration::from_secs(duration)) {
                    Ok(packet) => packet,
                    Err(_) => continue,
                };
                let addr_rcv = match packet.addr_rcv {
                    Some(addr) => addr,
                    None => {
                        log::error!("Field 'addr_rcv' not specified.");
                        continue;
                    }
                };

                // Connect and send
                let mut stream = match TcpStream::connect(&addr_rcv) {
                    Ok(stream) => stream,
                    Err(_) => {
                        log::error!("Cannot connect to address: {}", addr_rcv);
                        continue;
                    }
                };

                let a = packet.to_bytes();
                if let Err(err) = stream.write_all(a.as_slice()) {
                    log::error!("Cannot send to address: {} : {}", &addr_rcv, err);
                }
            }
        }))
    }

    pub fn send_file(&self, receiver_receiver2processor: &Receiver<Packet>, sender_processor2sender: &Sender<Packet>) {
        let addr_dns: SocketAddr = SocketAddr::new(IpAddr::V4(self.configs.ip_dns), self.configs.port_dns);
        let addr_current = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.configs.args.port));

        let mut packet: Packet;

        // 1. Read file
        log::debug!("1. Read file: {:?}", self.configs.args.path);

        let mut file = match File::open(self.configs.args.path.as_ref().unwrap()) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Reading file: {:?}. Got err: {}", self.configs.args.path, err);
                exit(1);
            }
        };
        let mut binary = Vec::new();
        if let Err(err) = file.read_to_end(&mut binary) {
            log::error!("Reading file: {:?}. Got err: {}", self.configs.args.path, err);
            exit(1);
        }

        // 2. Ask DNS for Master's address
        log::debug!("Ask Master address from DNS: {}", addr_dns);

        forward_packet(
            sender_processor2sender,
            Packet::create_ask_ip(addr_dns, addr_current.port()),
        );
        packet = wait_packet(receiver_receiver2processor);
        if PacketId::AskIpAck != packet.packet_id {
            log::error!("Must received AskIpAck from DNS. Got: {}", packet.packet_id);
            exit(1);
        }
        let addr_master = match packet.addr_master {
            Some(addr) => addr,
            None => {
                log::error!("Received packet not contain addr_master");
                exit(1);
            }
        };

        // 3. Connect to Master to get addr of node to send data
        log::debug!("Connect to Master: {}", addr_master);

        forward_packet(
            sender_processor2sender,
            Packet::create_request_from_client(
                Action::Write,
                self.configs.args.port,
                self.configs.args.name.as_ref().unwrap(),
                addr_master,
            ),
        );
        packet = wait_packet(receiver_receiver2processor);
        if PacketId::ResponseNodeIp != packet.packet_id {
            log::error!("Must received ResponseNodeIp from DNS. Got: {}", packet.packet_id);
            exit(1);
        }
        if packet.addr_data.is_none() {
            log::error!("Received packet not contained 'addr_data'");
            exit(1);
        };
        let addr_data = packet.addr_data.unwrap();

        // 4. Connect to Data node to send file
        log::debug!(
            "Connect to Data node to send file: {} - {:?}",
            addr_data,
            self.configs.args.path
        );

        forward_packet(
            sender_processor2sender,
            Packet::create_client_upload(addr_data, self.configs.args.name.as_ref().unwrap(), binary),
        );
    }
}
