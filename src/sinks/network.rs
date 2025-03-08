use std::{collections::VecDeque, io::Read as _, net::UdpSocket};

use super::Sink;
use anyhow::Result;
use log::warn;

pub struct UdpSinkPack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl UdpSinkPack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            socket,
            buffer: vec![0; buffer_size],
        }
    }
}

impl Sink for UdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        if data.is_empty() {
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

pub struct CheckedUdpSinkPack {
    pub current_id: u64,
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl CheckedUdpSinkPack {
    pub fn new(socket: UdpSocket, buffer_size: usize) -> Self {
        Self {
            current_id: 0,
            socket,
            buffer: vec![0; buffer_size],
        }
    }
}

impl Sink for CheckedUdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        if data.is_empty() {
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
