use std::{cmp::Ordering, collections::VecDeque, io::Write as _, net::UdpSocket};

use anyhow::Result;
use log::warn;

use crate::sources::Source;

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

impl Source for UdpSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
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

impl Source for CheckedUdpSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
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
