use std::{collections::VecDeque, net::UdpSocket};

use log::info;
use wasapi::{Direction, Handle, WaveFormat};

use anyhow::{Result, anyhow};

use crate::{device_utils, network, Args};

pub trait Source {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<usize>;
}

pub fn from_args(args: &Args) -> Result<(Box<dyn Source>, Option<Handle>)> {
    Ok(if let Some(address) = args.source.strip_prefix("udp://") {
        let socket = UdpSocket::bind(&address)?;
        let buffer_size = args.datagram_size;
        let pack = network::UdpSourcePack::new(socket, buffer_size);
        info!("Listening on {address} to packets of a most {buffer_size} bytes");
        (Box::new(pack), None)
    } else {
        let format = WaveFormat::new(args.bits_per_sample, args.bits_per_sample, &wasapi::SampleType::Int, args.sample_rate, args.channels, None);
        let device = device_utils::find_device_by_name(Direction::Capture, &args.source)?;
        let client = device_utils::open_device_with_format(&device, &format)?;
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
