use std::{
    collections::VecDeque,
    io::Read as _,
    net::UdpSocket,
    time::{Duration, Instant},
};

use crate::Restart;

use super::SendAudio;
use anyhow::{Result, anyhow};
use log::{debug, warn};

pub struct UdpSinkPack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl UdpSinkPack {
    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(address)?;
        Ok(Self {
            socket,
            buffer: vec![0; buffer_size],
        })
    }
}

impl Restart for UdpSinkPack {
    fn restart(&mut self) -> Result<()> {
        let remote_addr = self.socket.peer_addr()?;
        self.socket = UdpSocket::bind("0.0.0.0:0")?;
        self.socket.connect(remote_addr)?;
        Ok(())
    }
}

impl SendAudio for UdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let n_sent = usize::min(self.buffer.len(), data.len());
        if n_sent == self.buffer.len() {
            warn!("Splitting datagram!");
        }
        data.read_exact(&mut self.buffer[..n_sent])?;
        self.socket.send(&self.buffer[..n_sent])?;

        Ok(())
    }
}

pub struct CountedUdpSinkPack {
    pub current_id: u64,
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl CountedUdpSinkPack {
    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(address)?;
        Ok(Self {
            current_id: 0,
            socket,
            buffer: vec![0; buffer_size],
        })
    }
}

impl SendAudio for CountedUdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let tag = self.current_id.to_be_bytes();
        let n_sent = usize::min(self.buffer.len(), data.len() + tag.len());
        if n_sent == self.buffer.len() {
            warn!("Splitting datagram!");
        }
        self.buffer[..tag.len()].copy_from_slice(&tag);
        data.read_exact(&mut self.buffer[tag.len()..n_sent])?;
        self.socket.send(&self.buffer[..n_sent])?;

        self.current_id += 1;

        Ok(())
    }
}

impl Restart for CountedUdpSinkPack {
    fn restart(&mut self) -> Result<()> {
        self.current_id = 0;
        let remote_addr = self.socket.peer_addr()?;
        self.socket = UdpSocket::bind("0.0.0.0:0")?;
        self.socket.connect(remote_addr)?;
        Ok(())
    }
}

pub struct IdcSinkPack {
    address: socket2::SockAddr,
    socket: socket2::Socket,
    last_connection_attempt: Instant,
    buffer: Vec<u8>,
}

impl IdcSinkPack {
    fn create_socket(address: &socket2::SockAddr) -> Result<socket2::Socket> {
        let socket = socket2::Socket::new(
            if address.is_ipv4() {
                socket2::Domain::IPV4
            } else {
                socket2::Domain::IPV6
            },
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        socket.set_nonblocking(true)?;
        let _ = socket.connect(address);
        Ok(socket)
    }

    pub fn new(address: impl std::net::ToSocketAddrs, buffer_size: usize) -> Result<Self> {
        let address = address
            .to_socket_addrs()?
            .next()
            .ok_or(anyhow!("Couldn't get socket addr."))?
            .into();
        let socket = Self::create_socket(&address)?;
        Ok(Self {
            address,
            socket,
            last_connection_attempt: Instant::now(),
            buffer: vec![0; buffer_size],
        })
    }
}

impl SendAudio for IdcSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()> {
        let n_sent = usize::min(self.buffer.len(), data.len());
        if n_sent == self.buffer.len() {
            warn!("Splitting datagram!");
        }
        data.read_exact(&mut self.buffer[..n_sent])?;
        match self.socket.send(&self.buffer[..n_sent]) {
            Ok(_) => {}
            Err(error) => {
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    debug!("Encountered WouldBlock: {error:?}");
                } else if self.last_connection_attempt.elapsed() > Duration::from_millis(2000) {
                    debug!("Can't send so trying to reconnect");
                    match self.socket.connect(&self.address) {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        _ => self.socket = Self::create_socket(&self.address)?,
                    };
                    self.last_connection_attempt = Instant::now();
                }
                // Couldn't send, just consume the data
                data.clear();
            }
        };
        Ok(())
    }
}

impl Restart for IdcSinkPack {
    fn restart(&mut self) -> Result<()> {
        self.socket = Self::create_socket(&self.address)?;
        Ok(())
    }
}
