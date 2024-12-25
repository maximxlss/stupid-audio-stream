use wasapi::{AudioClient, Device, DeviceCollection, DeviceState, Direction, WaveFormat};

use anyhow::{Result, anyhow, bail};
use log::debug;

pub fn find_device_by_name(direction: Direction, query: &str) -> Result<Device> {
    let collection = DeviceCollection::new(&direction)
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

pub fn open_device_with_format(device: &Device, format: &WaveFormat) -> Result<AudioClient> {
    let state = device
        .get_state()
        .map_err(|err| anyhow!("Couldn't get device state due to error: {err}"))?;

    let DeviceState::Active = state else {
        bail!("Device is not active; it's state is {state}");
    };

    let mut client = device
        .get_iaudioclient()
        .map_err(|err| anyhow!("Couldn't get audio client due to error: {err}"))?;

    client
        .initialize_client(
            &format,
            0,
            &device.get_direction(),
            &client.get_sharemode().unwrap_or(wasapi::ShareMode::Shared),
            true,
        )
        .map_err(|err| anyhow!("Can't initialize client: {err}"))?;

    debug!("Opened device successfully");

    Ok(client)
}
