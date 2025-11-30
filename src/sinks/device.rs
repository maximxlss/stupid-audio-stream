use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use super::SendAudio;

pub struct DeviceSinkPack {
    pub client: wasapi::AudioClient,
    pub render_client: wasapi::AudioRenderClient,
    pub format: wasapi::WaveFormat,
}

impl SendAudio for DeviceSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()> {
        let mut frames_to_write =
            self.client
                .get_available_space_in_frames()
                .map_err(|err| anyhow!("Can't get available space: {err}"))? as usize;
        let blockalign = self.format.get_blockalign() as usize;
        if frames_to_write > data.len() / blockalign {
            frames_to_write = data.len() / blockalign;
        }
        if frames_to_write == 0 {
            return Ok(());
        }
        self.render_client
            .write_to_device_from_deque(frames_to_write, data, None)
            .map_err(|err| anyhow!("Couldn't write to device: {err}"))?;
        Ok(())
    }
}
