use log::debug;
use wasapi::{AudioClient, WaveFormat};

use anyhow::{Result, anyhow, bail};

pub fn find_device_by_name(direction: wasapi::Direction, query: &str) -> Result<wasapi::Device> {
    let enumerator = wasapi::DeviceEnumerator::new()?;
    let collection = enumerator.get_device_collection(&direction)
        .map_err(|err| anyhow!("Couldn't list devices for {direction:?} due to error: {err}"))?;

    let mut result = None;

    let query = query.to_lowercase();

    for device in &collection {
        let device = device.map_err(|err| anyhow!("Couldn't get device due to error: {err}"))?;

        let name = device
            .get_friendlyname()
            .map_err(|err| anyhow!("Couldn't get device name due to error: {err}"))?;

        if name.to_lowercase().contains(&query) {
            debug!("Found device {name:?} containing {query:?}");
            if result.is_some() {
                bail!("Multiple devices' name contains: {query:?}");
            }
            result = Some(device);
        }
    }

    if let Some(device) = result {
        Ok(device)
    } else {
        bail!("No device name contains: {query:?}");
    }
}

pub fn open_device_with_format(
    device: &wasapi::Device,
    format: &WaveFormat,
) -> Result<AudioClient> {
    let state = device
        .get_state()
        .map_err(|err| anyhow!("Couldn't get device state due to error: {err}"))?;

    let wasapi::DeviceState::Active = state else {
        bail!("Device is not active; it's state is {state}");
    };

    let mut client = device
        .get_iaudioclient()
        .map_err(|err| anyhow!("Couldn't get audio client due to error: {err}"))?;

    client
        .initialize_client(
            format,
            &device.get_direction(),
            &wasapi::StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: 100000,
            }, // TODO: figure out what to do with this buffer size
        )
        .map_err(|err| anyhow!("Can't initialize client: {err}"))?;

    debug!("Opened device successfully");

    Ok(client)
}
