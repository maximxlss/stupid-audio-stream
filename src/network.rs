use std::{cmp::Ordering, collections::VecDeque, io::{Read, Write}, net::UdpSocket};
use anyhow::Result;

use crate::{sinks::Sink, sources::Source};
use log::warn;


pub struct UdpSinkPack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl UdpSinkPack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            socket,
            buffer: vec![0; buffer_size]
        }
    }
}

impl Sink for UdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        let n_sent = usize::min(self.buffer.len(), data.len());
        if n_sent == self.buffer.len() {
            warn!("Splitting datagram!");
        }
        data.read_exact(&mut self.buffer[..n_sent])?;
        self.socket.send(&self.buffer[..n_sent])?;

        Ok(n_sent)
    }
}

pub struct UdpSourcePack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl UdpSourcePack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            socket,
            buffer: vec![0; buffer_size]
        }
    }
}

impl Source for UdpSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let (n_read, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
        buf.write(&self.buffer[..n_read])?;
        Ok(n_read)
    }
}


type IdSize = u64;

pub struct CheckedUdpSinkPack {
    pub current_id: IdSize,
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl CheckedUdpSinkPack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            current_id: 0,
            socket,
            buffer: vec![0; buffer_size]
        }
    }
}

impl Sink for CheckedUdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
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

        Ok(n_sent)
    }
}

pub struct CheckedUdpSourcePack {
    pub current_id: IdSize,
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl CheckedUdpSourcePack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            current_id: 0,
            socket,
            buffer: vec![0; buffer_size]
        }
    }
}

impl Source for CheckedUdpSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let (n_read, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
        let tag_size = self.current_id.to_be_bytes().len();
        let supposed_id = IdSize::from_be_bytes(self.buffer[..tag_size].try_into().unwrap());
        match supposed_id.cmp(&self.current_id) {
            Ordering::Less => {
                warn!("Got a packet from the past, {} packets late", self.current_id - supposed_id);
                self.current_id = supposed_id;
            },
            Ordering::Greater => {
                warn!("Got a packet from the future, {} packets early", supposed_id - self.current_id);
                self.current_id = supposed_id;
            },
            Ordering::Equal => {}
        }
        buf.write(&self.buffer[tag_size..n_read])?;

        self.current_id += 1;

        Ok(n_read)
    }
}
