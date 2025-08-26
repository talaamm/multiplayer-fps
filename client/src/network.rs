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
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(&server_addr)?;
        socket.set_nonblocking(true)?;

        let (tx_outgoing, rx_outgoing) = channel::<ClientToServer>();
        let (tx_incoming, rx_incoming) = channel::<ServerToClient>();

        // Spawn network thread
        thread::spawn(move || {
            // Send initial Join
            let join = ClientToServer::Join(protocol::JoinRequest { username });
            if let Ok(bytes) = protocol::encode_client(&join) {
                let _ = socket.send(&bytes);
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
                            let _ = socket.send(&bytes);
                            last_send = Instant::now();
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                // Pump incoming
                match socket.recv(&mut buf) {
                    Ok(len) => {
                        if let Ok(msg) = protocol::decode_server(&buf[..len]) {
                            let _ = tx_incoming.send(msg);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        Ok(Self { tx_outgoing, rx_incoming })
    }
}


