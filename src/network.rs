use std::{collections::VecDeque, io::{Read, Write}, net::UdpSocket};
use anyhow::Result;

use crate::{sinks::Sink, sources::Source};


pub struct UdpSinkPack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl Sink for UdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        let n_sent = usize::min(self.buffer.len(), data.len());
        data.read_exact(&mut self.buffer[..n_sent])?;
        self.socket.send(&self.buffer[..n_sent])?;

        Ok(n_sent)
    }
}

pub struct UdpSourcePack {
    pub socket: UdpSocket,
    pub buffer: Vec<u8>,
}

impl Source for UdpSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let (n_read, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
        buf.write(&self.buffer[..n_read])?;
        Ok(n_read)
    }
}
