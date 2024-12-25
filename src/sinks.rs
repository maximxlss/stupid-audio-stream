use std::{collections::VecDeque, io::Read, net::UdpSocket};

use log::info;
use wasapi::{AudioClient, AudioRenderClient, Direction, Handle, WaveFormat};

use anyhow::{Result, anyhow};

use crate::{DEFAULT_FORMAT, MAX_DATAGRAM, find_device_by_name, open_device_with_format};

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

struct UdpSinkPack {
    socket: UdpSocket,
    buffer: Vec<u8>,
}

impl Sink for UdpSinkPack {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize> {
        let n_sent = usize::min(self.buffer.len(), data.len());
        data.read_exact(&mut self.buffer[..n_sent])?;
        self.socket.send(&self.buffer[..n_sent])?;

        Ok(n_sent)
    }
}

pub fn get_sink_from_string(query: &str) -> Result<(Box<dyn Sink>, Option<Handle>)> {
    Ok(if let Some(address) = query.strip_prefix("udp://") {
        let socket = UdpSocket::bind("0.0.0.0:13371")?;
        socket.connect(address)?;
        let buffer_size = MAX_DATAGRAM.load(std::sync::atomic::Ordering::Relaxed);
        let buffer = vec![0u8; buffer_size];
        let pack = UdpSinkPack {socket, buffer};
        info!("Sending to {address} datagrams of up to {buffer_size} bytes");
        (Box::new(pack), None)
    } else {
        let device = find_device_by_name(Direction::Render, &query)?;
        let client = open_device_with_format(&device, &DEFAULT_FORMAT)?;
        let render_client = client
            .get_audiorenderclient()
            .map_err(|err| anyhow!("Can't get the capture client for device: {err}"))?;
        let event_handle = client
            .set_get_eventhandle()
            .map_err(|err| anyhow!("Couldn't get event handle of device: {err}"))?;
        client
            .start_stream()
            .map_err(|err| anyhow!("Couldn't start stream of device: {err}"))?;

        let name = device
            .get_friendlyname()
            .map_err(|err| anyhow!("Couldn't get device name due to error: {err}"))?;
        info!("Sending to {name}");
        (
            Box::new(DeviceSinkPack {
                render_client,
                client,
                format: DEFAULT_FORMAT.clone(),
            }),
            Some(event_handle),
        )
    })
}
