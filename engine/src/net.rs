use std::net::UdpSocket;
use std::thread;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use logging::error;
use logging::info;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::Lazy;

pub trait Packet: Serialize + DeserializeOwned + Send + 'static {
    const ID: u16;

    fn encode(&self, buf: &mut Vec<u8>) {
        match postcard::to_extend(self, core::mem::take(buf)) {
            Ok(b) => *buf = b,
            Err(e) => logging::error!("Failed to encode packet {}: {}", Self::ID, e),
        }
    }

    fn decode(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        postcard::from_bytes(bytes).ok()
    }
}

pub struct Net {
    rx: Receiver<(u16, Vec<u8>)>,
    tx: Option<Sender<(u16, Vec<u8>)>>,
    socket: Lazy<UdpSocket>,
    messages: Vec<(u16, Vec<u8>)>,
    send_buf: Vec<u8>,
}

impl Net {
    pub(crate) fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<(u16, Vec<u8>)>();

        Self {
            rx,
            tx: Some(tx),
            socket: Lazy::empty(),
            messages: Vec::new(),
            send_buf: Vec::new(),
        }
    }

    pub fn connect<A: AsRef<str>>(&mut self, addr: A) {
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to udp socket: {}", e);
                return;
            }
        };

        let recv_socket = socket.try_clone().expect("clone udp socket");
        self.socket.set(socket);

        if let Err(e) = recv_socket.connect(addr.as_ref()) {
            error!("Failed to connect udp socket: {}", e);
            return;
        }

        info!("Started listening to udp socket.");

        let tx = self.tx.take().unwrap();

        thread::spawn(move || {
            let mut buf = [0u8; 65535];

            loop {
                match recv_socket.recv(&mut buf) {
                    Ok(len) if len >= 2 => {
                        let id = u16::from_le_bytes([buf[0], buf[1]]);

                        if tx.send((id, buf[2..len].to_vec())).is_err() {
                            break;
                        }
                    }

                    Ok(_) => { /* runt packet, ignore */ }

                    Err(e) => {
                        error!("Failed receiving from the udp socket: {}", e);
                        break;
                    }
                }
            }
        });
    }

    pub fn send<P: Packet>(&mut self, packet: &P) {
        self.send_buf.clear();
        self.send_buf.extend_from_slice(&P::ID.to_le_bytes());

        packet.encode(&mut self.send_buf);

        if let Err(e) = self.socket.send(&self.send_buf) {
            error!("Failed to send: {}", e);
        }
    }

    pub fn read<P: Packet>(&self) -> impl Iterator<Item = P> + '_ {
        self.messages
            .iter()
            .filter(|(id, _)| *id == P::ID)
            .filter_map(|(_, bytes)| P::decode(bytes))
    }

    /// Drain everything the socket thread has pushed since last frame.
    /// Call once per frame, before scene updates.
    pub(crate) fn poll(&mut self) {
        // try_iter never blocks; it yields only what's already queued
        self.messages.extend(self.rx.try_iter());
    }

    pub fn messages<P: Packet>(&self) -> impl Iterator<Item = P> + '_ {
        self.messages
            .iter()
            .filter(|(id, _)| *id == P::ID)
            .filter_map(|(_, bytes)| P::decode(bytes))
    }

    pub(crate) fn flush(&mut self) {
        self.messages.clear();
    }
}
