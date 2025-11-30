use std::collections::VecDeque;

use anyhow::{Result, anyhow};

use crate::{Restart, device_utils, sources::RecvAudio};

pub struct DeviceSourcePack {
    device: wasapi::Device,
    format: wasapi::WaveFormat,
    audio_client: wasapi::AudioClient,
    audio_capture_client: wasapi::AudioCaptureClient,
    event_handle: wasapi::Handle,
}

impl DeviceSourcePack {
    pub fn new(device: wasapi::Device, format: wasapi::WaveFormat) -> Result<Self> {
        let audio_client = device_utils::open_device_with_format(&device, &format)?;
        let audio_capture_client = audio_client
            .get_audiocaptureclient()
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
            audio_capture_client,
            event_handle,
        })
    }

    pub fn device(&self) -> &wasapi::Device {
        &self.device
    }

    fn maybe_recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize> {
        let prev = buf.len();
        self.audio_capture_client
            .read_from_device_to_deque(buf)
            .map_err(|err| anyhow!("Couldn't read from device: {err}"))?;
        Ok(buf.len() - prev)
    }
}

impl RecvAudio for DeviceSourcePack {
    fn recv_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()> {
        while let 0 = self.maybe_recv_to_deque(buf)? {
            self.event_handle
                .wait_for_event(1000)
                .map_err(|err| anyhow!("Timeout error: {err}"))?;
        }
        Ok(())
    }
}

impl Restart for DeviceSourcePack {
    fn restart(&mut self) -> Result<()> {
        self.audio_client
            .stop_stream()
            .map_err(|err| anyhow!("Couldn't stop stream of device: {err}"))?;

        self.audio_client = device_utils::open_device_with_format(&self.device, &self.format)?;
        self.audio_capture_client = self
            .audio_client
            .get_audiocaptureclient()
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
