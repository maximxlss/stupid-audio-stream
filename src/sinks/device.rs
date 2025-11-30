use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::{Restart, device_utils};

use super::SendAudio;

pub struct DeviceSinkPack {
    device: wasapi::Device,
    format: wasapi::WaveFormat,
    audio_client: wasapi::AudioClient,
    audio_render_client: wasapi::AudioRenderClient,
    event_handle: wasapi::Handle,
}

impl DeviceSinkPack {
    pub fn new(device: wasapi::Device, format: wasapi::WaveFormat) -> Result<Self> {
        let audio_client = device_utils::open_device_with_format(&device, &format)?;
        let audio_render_client = audio_client
            .get_audiorenderclient()
            .map_err(|err| anyhow!("Can't get the capture client for device: {err}"))?;
        let event_handle = audio_client
            .set_get_eventhandle()
            .map_err(|err| anyhow!("Couldn't get event handle of device: {err}"))?;
        audio_client
            .start_stream()
            .map_err(|err| anyhow!("Couldn't start stream of device: {err}"))?;

        Ok(Self {
            device,
            format,
            audio_client,
            audio_render_client,
            event_handle,
        })
    }

    pub fn device(&self) -> &wasapi::Device {
        &self.device
    }
}

impl SendAudio for DeviceSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()> {
        let mut frames_to_write =
            self.audio_client
                .get_available_space_in_frames()
                .map_err(|err| anyhow!("Can't get available space: {err}"))? as usize;
        let blockalign = self.format.get_blockalign() as usize;
        if frames_to_write > data.len() / blockalign {
            frames_to_write = data.len() / blockalign;
        }
        if frames_to_write == 0 {
            return Ok(());
        }
        self.audio_render_client
            .write_to_device_from_deque(frames_to_write, data, None)
            .map_err(|err| anyhow!("Couldn't write to device: {err}"))?;
        Ok(())
    }
}

impl Restart for DeviceSinkPack {
    fn restart(&mut self) -> Result<()> {
        self.audio_client
            .stop_stream()
            .map_err(|err| anyhow!("Couldn't stop stream of device: {err}"))?;

        self.audio_client = device_utils::open_device_with_format(&self.device, &self.format)?;
        self.audio_render_client = self
            .audio_client
            .get_audiorenderclient()
            .map_err(|err| anyhow!("Can't get the capture client for device: {err}"))?;
        self.event_handle = self
            .audio_client
            .set_get_eventhandle()
            .map_err(|err| anyhow!("Couldn't get event handle of device: {err}"))?;
        self.audio_client
            .start_stream()
            .map_err(|err| anyhow!("Couldn't start stream of device: {err}"))?;

        Ok(())
    }
}
