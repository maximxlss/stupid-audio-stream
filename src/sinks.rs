use std::{collections::VecDeque, io::Read, net::UdpSocket};

use wasapi::{
    AudioClient, AudioRenderClient, WaveFormat
};

use anyhow::{Result, anyhow};

use crate::MAX_DATAGRAM;

pub trait Sink {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize>;
}

pub struct DeviceSinkPack {
    pub client: AudioClient,
    pub render_client: AudioRenderClient,
    pub format: WaveFormat,
}

impl Sink for DeviceSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        let mut frames_to_write =
            self.client
                .get_available_space_in_frames()
                .map_err(|err| anyhow!("Can't get available space: {err}"))? as usize;
        let blockalign = self.format.get_blockalign() as usize;
        if frames_to_write as usize > data.len() / blockalign {
            frames_to_write = data.len() / blockalign;
        }
        if frames_to_write == 0 {
            return Ok(0);
        }
        let len_before = data.len();
        self.render_client
            .write_to_device_from_deque(frames_to_write, data, None)
            .map_err(|err| anyhow!("Couldn't write to device: {err}"))?;
        let n_sent = len_before - data.len();
        return Ok(n_sent);
    }
}



impl Sink for UdpSocket {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        let mut frame = [0u8; MAX_DATAGRAM];

        let n_sent = usize::min(MAX_DATAGRAM, data.len());
        data.read_exact(&mut frame[..n_sent])?;
        self.send(&frame[..n_sent])?;

        Ok(n_sent)
    }
}
