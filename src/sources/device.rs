use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::sources::RecvAudio;

pub type DeviceSourcePack = wasapi::AudioCaptureClient;

impl RecvAudio for DeviceSourcePack {
    fn recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
        self.read_from_device_to_deque(buf)
            .map_err(|err| anyhow!("Couldn't read from device: {err}"))?;
        Ok(())
    }
}
