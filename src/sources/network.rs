use std::{
    cmp::Ordering,
    collections::VecDeque,
    io::{Read as _, Write as _},
    net::UdpSocket,
};

use anyhow::{Result, anyhow};
use log::{debug, warn};

use crate::sources::RecvAudio;

pub struct UdpSourcePack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl UdpSourcePack {
    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(address)?,
            buffer: vec![0; buffer_size],
        })
    }
}

impl RecvAudio for UdpSourcePack {
    fn recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
        let (n_read, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
        buf.write_all(&self.buffer[..n_read])?;
        Ok(())
    }
}

pub struct CheckedUdpSourcePack {
    pub current_id: u64,
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl CheckedUdpSourcePack {
    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        Ok(Self {
            current_id: 0,
            socket: UdpSocket::bind(address)?,
            buffer: vec![0; buffer_size],
        })
    }
}

impl RecvAudio for CheckedUdpSourcePack {
    fn recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
        let (n_read, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
        let tag_size = self.current_id.to_be_bytes().len();
        let supposed_id = u64::from_be_bytes(self.buffer[..tag_size].try_into().unwrap());
        match supposed_id.cmp(&self.current_id) {
            Ordering::Less => {
                warn!(
                    "Got a packet from the past, {} packets late",
                    self.current_id - supposed_id
                );
                self.current_id = supposed_id;
            }
            Ordering::Greater => {
                warn!(
                    "Got a packet from the future, {} packets early",
                    supposed_id - self.current_id
                );
                self.current_id = supposed_id;
            }
            Ordering::Equal => {}
        }
        buf.write_all(&self.buffer[tag_size..n_read])?;

        self.current_id += 1;

        Ok(())
    }
}

pub struct IdcSourcePack {
    listener: socket2::Socket,
    socket: Option<socket2::Socket>,
    buffer: Vec<u8>,
}

impl IdcSourcePack {
    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        let address = address
            .to_socket_addrs()?
            .next()
            .ok_or(anyhow!("Couldn't get socket addr."))?;
        let listener = socket2::Socket::new(
            if address.is_ipv4() {
                socket2::Domain::IPV4
            } else {
                socket2::Domain::IPV6
            },
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        let address = address.into();
        listener.set_nonblocking(true)?;
        listener.bind(&address)?;
        listener.listen(1)?;
        Ok(Self {
            listener,
            socket: None,
            buffer: vec![0; buffer_size],
        })
    }
}

impl RecvAudio for IdcSourcePack {
    fn recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
        match &mut self.socket {
            None => {
                if let Ok((s, addr)) = self.listener.accept() {
                    self.socket = Some(s);
                    debug!("Accepted connection from {:?}", addr);
                }
            }
            Some(s) => match s.read(self.buffer.as_mut_slice()) {
                Ok(n_read) => buf.write_all(&self.buffer[..n_read])?,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                _ => {
                    self.socket = None;
                    debug!("Connection dropped");
                }
            },
        }

        Ok(())
    }
}
