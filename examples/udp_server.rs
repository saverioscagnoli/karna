use std::collections::HashSet;
use std::net::SocketAddr;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5000")?;
    println!("UDP relay listening on port 5000");

    let mut clients: HashSet<SocketAddr> = HashSet::new();
    let mut buf = [0u8; 65535];

    loop {
        let (len, src) = match socket.recv_from(&mut buf) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("recv error: {e}");
                continue; // don't kill the server over one bad recv
            }
        };

        // first packet from a new address registers them
        if clients.insert(src) {
            println!("new client: {src} ({} total)", clients.len());
        }

        // relay to everyone except the sender
        for &client in &clients {
            if client != src {
                if let Err(e) = socket.send_to(&buf[..len], client) {
                    eprintln!("send to {client} failed: {e}");
                }
            }
        }
    }
}
