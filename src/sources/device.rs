use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::sources::Source;

pub type DeviceSourcePack = wasapi::AudioCaptureClient;

impl Source for DeviceSourcePack {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let len_before = buf.len();
        self.read_from_device_to_deque(buf)
            .map_err(|err| anyhow!("Couldn't read from device: {err}"))?;
        let n_read = buf.len() - len_before;
        Ok(n_read)
    }
}
