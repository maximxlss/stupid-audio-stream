use std::{collections::VecDeque, io::Write, net::UdpSocket};

use log::info;
use wasapi::{AudioCaptureClient, Direction, Handle};

use anyhow::{Result, anyhow};

use crate::{DEFAULT_FORMAT, MAX_DATAGRAM, find_device_by_name, open_device_with_format};

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

pub fn get_source_from_string(query: &str) -> Result<(Box<dyn Source>, Option<Handle>)> {
    Ok(if let Some(address) = query.strip_prefix("udp://") {
        let socket = UdpSocket::bind(&address)?;
        info!("Listening on {address}");
        (Box::new(socket), None)
    } else {
        let device = find_device_by_name(Direction::Capture, &query)?;
        let client = open_device_with_format(&device, &DEFAULT_FORMAT)?;
        let capture_client = client
            .get_audiocaptureclient()
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
        info!("Capturing from {name}");
        (Box::new(capture_client), Some(event_handle))
    })
}
