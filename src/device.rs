use std::collections::VecDeque;

use wasapi::{AudioCaptureClient, AudioClient, AudioRenderClient, WaveFormat};

use anyhow::{Result, anyhow};

use crate::{sinks::Sink, sources::Source};

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

impl Source for AudioCaptureClient {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let len_before = buf.len();
        self.read_from_device_to_deque(buf)
            .map_err(|err| anyhow!("Couldn't read from device: {err}"))?;
        let n_read = buf.len() - len_before;
        return Ok(n_read);
    }
}
