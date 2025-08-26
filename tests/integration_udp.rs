use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use bincode;
use multiplayer_fps::net::{ClientMsg, ServerMsg};

#[test]
fn client_server_handshake() {
   
    thread::spawn(|| {
        multiplayer_fps::server::run("127.0.0.1:4000").unwrap();
    });

   
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.connect("127.0.0.1:4000").unwrap();

   
    let hello = ClientMsg::Hello { name: "test".into() };
    client.send(&bincode::serialize(&hello).unwrap()).unwrap();

   
    client.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    let mut buf = [0u8; 1500];
    let len = client.recv(&mut buf).unwrap();

    
    let resp: ServerMsg = bincode::deserialize(&buf[..len]).unwrap();

   
    match resp {
        ServerMsg::Welcome { player_id } => assert!(player_id > 0),
        _ => panic!("Expected Welcome message"),
    }
}
