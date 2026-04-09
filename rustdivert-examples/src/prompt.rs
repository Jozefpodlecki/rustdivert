use std::fmt;

use anyhow::Result;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use rustdivert::WinDivertParam;

#[derive(Debug, Clone, Copy)]
pub enum ParamKind {
    QueueLength,
    QueueSize,
    QueueTime,
}

impl ParamKind {
    pub fn as_raw(self) -> WinDivertParam {
        match self {
            ParamKind::QueueLength => WinDivertParam::QueueLength,
            ParamKind::QueueSize   => WinDivertParam::QueueSize,
            ParamKind::QueueTime   => WinDivertParam::QueueTime,
        }
    }
}

pub struct ParamMeta {
    pub name: &'static str,
    pub unit: &'static str,
    pub min: u64,
    pub max: u64,
    pub default: u64,
}

impl ParamKind {
    pub fn meta(self) -> ParamMeta {
        match self {
            ParamKind::QueueLength => ParamMeta {
                name: "Queue Length",
                unit: "packets",
                min: 32,
                max: 16384,
                default: 4096,
            },
            ParamKind::QueueTime => ParamMeta {
                name: "Queue Time",
                unit: "ms",
                min: 100,
                max: 16000,
                default: 2000,
            },
            ParamKind::QueueSize => ParamMeta {
                name: "Queue Size",
                unit: "bytes",
                min: 65535,
                max: 33_554_432,
                default: 4_194_304,
            },
        }
    }
}

impl fmt::Display for ParamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let meta = self.meta();
        write!(f, "{} ({})", meta.name, meta.unit)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    ClientServer,
    RustDivert,
    GetSetParams,
    Continue,
    Exit,
}

impl fmt::Display for AppMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AppMode::ClientServer => "Run client/server example",
            AppMode::RustDivert => "Run rustdivert packet sniffer",
            AppMode::GetSetParams => "Run rustdivert example where it sets/gets params",
            AppMode::Exit => "Exit",
            AppMode::Continue => "Continue",
        };
        write!(f, "{s}")
    }
}

pub fn prompt_mode() -> Result<AppMode> {
    let options = [
        AppMode::ClientServer,
        AppMode::RustDivert,
        AppMode::GetSetParams,
        AppMode::Exit,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose what to run")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(options[selection])
}

pub fn prompt_port() -> Result<u16> {
    loop {
        let input: String = Input::new()
            .with_prompt("Enter port")
            .default("53124".into())
            .interact_text()?;

        match input.parse::<u16>() {
            Ok(port) => return Ok(port),
            Err(_) => println!("Invalid port, try again."),
        }
    }
}

pub fn prompt_ip() -> Result<String> {
    let options = ["127.0.0.1", "Custom"];

    let selection = Select::new()
        .with_prompt("Select IP address")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok("127.0.0.1".into()),
        _ => {
            let ip: String = Input::new()
                .with_prompt("Enter IP address")
                .interact_text()?;
            Ok(ip)
        }
    }
}


pub fn prompt_filter(port: u16) -> Result<String> {
    loop {
        let default = format!(
            "ip && tcp && loopback && (tcp.SrcPort == {} || tcp.DstPort == {})",
            port, port
        );

        let filter: String = Input::new()
            .with_prompt("Enter WinDivert filter")
            .default(default.clone())
            .interact_text()?;

            
        if filter.trim().is_empty() {
            println!("Filter cannot be empty.");
            continue;
        }

        return Ok(filter);
    }
}

pub fn prompt_param() -> Result<ParamKind> {
    let params = [
        ParamKind::QueueLength,
        ParamKind::QueueSize,
        ParamKind::QueueTime,
    ];

    let selection = Select::new()
        .with_prompt("Select parameter")
        .items(&params)
        .interact()?;

    Ok(params[selection])
}

pub fn prompt_param_value(
    param: ParamKind,
    current: u64,
) -> Result<u64> {
    let meta = param.meta();

    loop {
        let input: String = Input::new()
            .with_prompt(format!(
                "{} (current: {}, default: {}, range: {}–{} {})",
                meta.name,
                current,
                meta.default,
                meta.min,
                meta.max,
                meta.unit
            ))
            .default(current.to_string())
            .interact_text()?;

        match input.parse::<u64>() {
            Ok(v) if v >= meta.min && v <= meta.max => return Ok(v),
            _ => println!(
                "Invalid value. Must be between {} and {} {}",
                meta.min, meta.max, meta.unit
            ),
        }
    }
}