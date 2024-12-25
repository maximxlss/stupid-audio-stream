use std::{collections::VecDeque, io::Write, net::UdpSocket};

use wasapi::AudioCaptureClient;

use anyhow::{Result, anyhow};


use crate::MAX_DATAGRAM;

pub trait Source {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize>;
}

impl Source for AudioCaptureClient {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let len_before = buf.len();
        self.read_from_device_to_deque(buf)
            .map_err(|err| anyhow!("Couldn't read from device: {err}"))?;
        let n_read = buf.len() - len_before;
        return Ok(n_read);
    }
}

impl Source for UdpSocket {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let mut frame = [0u8; MAX_DATAGRAM];
        let (n_read, _) = self.recv_from(&mut frame)?;
        buf.write(&frame[..n_read])?;
        Ok(n_read)
    }
}