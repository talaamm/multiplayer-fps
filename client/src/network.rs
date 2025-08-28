use std::net::UdpSocket;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use protocol::{self, ClientToServer, ServerToClient};

pub struct NetClient {
    pub tx_outgoing: Sender<ClientToServer>,
    pub rx_incoming: Receiver<ServerToClient>,
}

impl NetClient {
    pub fn start(server_addr: String, username: String) -> std::io::Result<Self> {
        println!("[client] binding UDP socket on 0.0.0.0:0 ...");
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        println!("[client] local addr = {:?}", socket.local_addr());
        println!("[client] connecting UDP to {} ...", server_addr);
        socket.connect(&server_addr)?;
        println!("[client] connected. setting nonblocking.");
        socket.set_nonblocking(true)?;

        let (tx_outgoing, rx_outgoing) = channel::<ClientToServer>();
        let (tx_incoming, rx_incoming) = channel::<ServerToClient>();

        // Spawn network thread
        thread::spawn(move || {
            // Send initial Join
            let join = ClientToServer::Join(protocol::JoinRequest { username });
            match protocol::encode_client(&join) {
                Ok(bytes) => {
                    println!("[client->server] sending Join ({} bytes)", bytes.len());
                    match socket.send(&bytes) {
                        Ok(n) => println!("[client->server] sent {} bytes", n),
                        Err(e) => println!("[client->server][send_error] {}", e),
                    }
                }
                Err(e) => println!("[client][encode_error] {}", e),
            }

            let mut buf = vec![0u8; 64 * 1024];
            let mut last_send = Instant::now();
            let min_send_dt = Duration::from_millis(15); // ~66 Hz cap

            loop {
                // Pump outgoing
                match rx_outgoing.try_recv() {
                    Ok(msg) => {
                        if last_send.elapsed() < min_send_dt { /* rate limit */ }
                        if let Ok(bytes) = protocol::encode_client(&msg) {
                            match socket.send(&bytes) {
                                Ok(n) => {
                                    println!("[client->server] sent {} bytes", n);
                                    last_send = Instant::now();
                                }
                                Err(e) => println!("[client->server][send_error] {}", e),
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                // Pump incoming
                match socket.recv(&mut buf) {
                    Ok(len) => {
                        println!("[server->client] recv {} bytes", len);
                        if let Ok(msg) = protocol::decode_server(&buf[..len]) {
                            let _ = tx_incoming.send(msg);
                        } else {
                            println!("[server->client][decode_error]");
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => {
                        println!("[client][recv_error] {}", e);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        Ok(Self { tx_outgoing, rx_incoming })
    }
}


