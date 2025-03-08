use std::{collections::VecDeque, net::UdpSocket};

use log::info;
use wasapi::{Direction, Handle, WaveFormat};

use anyhow::{Result, anyhow};

use crate::{Args, device_utils};

pub mod device;
pub mod network;

pub trait Sink {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<usize>;
}

pub fn from_args(args: &Args) -> Result<(Box<dyn Sink>, Option<Handle>)> {
    Ok(if let Some(address) = args.sink.strip_prefix("udp://") {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(address)?;
        let buffer_size = args.datagram_size;
        if args.checked_udp {
            let pack = network::CheckedUdpSinkPack::new(socket, buffer_size);
            info!("Sending to {address} datagrams of up to {buffer_size} bytes with loss checks");
            (Box::new(pack), None)
        } else {
            let pack = network::UdpSinkPack::new(socket, buffer_size);
            info!("Sending to {address} datagrams of up to {buffer_size} bytes");
            (Box::new(pack), None)
        }
    } else {
        let format = WaveFormat::new(
            args.bits_per_sample,
            args.bits_per_sample,
            &wasapi::SampleType::Int,
            args.sample_rate,
            args.channels,
            None,
        );
        let device = device_utils::find_device_by_name(Direction::Render, &args.sink)?;
        let client = device_utils::open_device_with_format(&device, &format)?;
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
            Box::new(device::DeviceSinkPack {
                render_client,
                client,
                format,
            }),
            Some(event_handle),
        )
    })
}
