use std::collections::VecDeque;

use log::info;
use wasapi::{Direction, WaveFormat};

use anyhow::{Result, anyhow};

use crate::{Args, SendAudioRestart, device_utils};

pub mod device;
pub mod network;

pub trait SendAudio {
    fn send_from_deque(&mut self, data: &mut VecDeque<u8>) -> Result<()>;
}

pub fn from_args(args: &Args) -> Result<Box<dyn SendAudioRestart>> {
    Ok(if let Some(address) = args.sink.strip_prefix("udp://") {
        let buffer_size = args.datagram_size;
        if args.counted_udp {
            let pack = network::CountedUdpSinkPack::new(address, buffer_size)?;
            info!("Sending to {address} datagrams of up to {buffer_size} bytes with loss checks");
            Box::new(pack)
        } else {
            let pack = network::UdpSinkPack::new(address, buffer_size)?;
            info!("Sending to {address} datagrams of up to {buffer_size} bytes");
            Box::new(pack)
        }
    } else if let Some(address) = args.sink.strip_prefix("idc://") {
        let buffer_size = args.datagram_size;
        let pack = network::IdcSinkPack::new(address, buffer_size)?;
        info!("Sending to {address} datagrams of up to {buffer_size} bytes without caring");
        Box::new(pack)
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
        let sink_pack = device::DeviceSinkPack::new(device, format)?;

        let name = sink_pack
            .device()
            .get_friendlyname()
            .map_err(|err| anyhow!("Couldn't get device name due to error: {err}"))?;
        info!("Sending to {name}");

        Box::new(sink_pack)
    })
}
