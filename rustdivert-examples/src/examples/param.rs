use anyhow::Result;
use log::*;
use rustdivert::{WinDivertFlags, WinDivertLayer, WinDivertParam, WindivertOptions, sync::Windivert};

use crate::prompt::*;

pub fn create_basic_windivert() -> Result<Windivert> {
    let options = WindivertOptions::default();
    let priority = 0;
    let filter = "ip && tcp";
    let flags = WinDivertFlags::new().set_sniff().set_recv_only();
    let layer = WinDivertLayer::Network;
    let windivert = Windivert::open(
        options,
        layer, 
        &filter, 
        priority, 
        flags
    )?;

    Ok(windivert)
}

pub fn modify_param(windivert: &Windivert) -> Result<()> {
    let param = prompt_param()?;
    let current = windivert.get_param(param.as_raw())?;
    let value = prompt_param_value(param, current)?;

    windivert.set_param(param.as_raw(), value)?;

    info!("Updated {:?} to {}", param, value);

    Ok(())
}

pub fn print_params(windivert: &Windivert) -> Result<()> {

    let params = [
        WinDivertParam::QueueLength,
        WinDivertParam::QueueSize,
        WinDivertParam::QueueTime,
        WinDivertParam::VersionMajor,
        WinDivertParam::VersionMinor,
    ];

    info!("Getting current params");
    for param in params {
        let value = windivert.get_param(param)?;
        info!("{:?} = {}", param, value);
    }

    Ok(())
}