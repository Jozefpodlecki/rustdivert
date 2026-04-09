
use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use etherparse::{PacketHeaders, SlicedPacket};
use rustdivert::{WinDivertFlags, WinDivertPacket, WindivertOptions, sync::Windivert};
use tokio::{signal, sync::{Mutex, watch::{self, Receiver}}, task::{JoinHandle, spawn_blocking}, time::sleep};
use crate::{client::Client, server::Server};
use log::*;

pub fn setup_client_server(port: u16) -> Result<()> {
    let mut server = Server::new();

    let addr = format!("127.0.0.1:{port}");
    server.start(addr.clone())?;

    let mut client = Client::new();
    client.start(addr);

    Ok(())
}

pub async fn run_using_rustdivert() -> Result<()> {
    info!("Running example using rustdivert");

    let port = 53124;
    setup_client_server(port)?;

    // let filter = format!("ip && tcp");
    // let filter = format!(
    //     "ip && tcp && (tcp.SrcPort == {} || tcp.DstPort == {}) && ip.SrcAddr == 127.0.0.1 && ip.DstAddr == 127.0.0.1",
    //     port, port
    // );
     let filter = format!("ip && tcp && loopback && (tcp.SrcPort == {} || tcp.DstPort == {})", port, port);
    
    let priority = 0;
    let flags = WinDivertFlags::new().set_sniff();
    let options = WindivertOptions {
        install_service_on_file_not_found: false,
    };
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let windivert = Windivert::open(
        options,
        rustdivert::WinDivertLayer::Network, 
        &filter, 
        priority, 
        flags
    )?;
   
    let ctrl_c_task = tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down...");
                shutdown_tx.send(true);
            }
            Err(e) => {
                error!("Unable to listen for shutdown signal: {}", e);
            }
        }
    });


    let processing_task: JoinHandle<std::result::Result<(), _>> = tokio::spawn(async move {
        on_process(windivert, shutdown_rx).await
    });

    tokio::select! {
        _ = ctrl_c_task => {},
        _ = processing_task => {},
    }

    Ok(())
}

pub async fn on_process(windivert: Windivert, mut shutdown_rx: Receiver<bool>) -> Result<()> {
    let windivert = Arc::new(windivert);
    
    loop {
        let windivert = windivert.clone();

        let task = spawn_blocking(move || {
            windivert.recv()
        });

        tokio::select! {
            _ = shutdown_rx.changed() => {
                info!("Shutdown signal received");
                break;
            },
            result = task => {
                match result {
                    Ok(result) => match result {
                        Ok(packet) => on_receive_using_rustdivert(packet)?,
                        Err(_) => {},
                    },
                    Err(err) => {
                        error!("Receive error: {}", err);
                        break;
                    }
                };
            },
        }
    }

    anyhow::Ok(())
}

pub fn on_receive_using_rustdivert(packet: WinDivertPacket) -> Result<()> {
    let sliced = SlicedPacket::from_ip(&packet.data)?;
    let headers = PacketHeaders::from_ip_slice(&packet.data)?;
    
    if let Some(net) = headers.net {
        match net {
            etherparse::NetHeaders::Ipv4(ip4, _) => {
                let src_ip = std::net::Ipv4Addr::from(ip4.source);
                let dst_ip = std::net::Ipv4Addr::from(ip4.destination);
                println!("{} -> {}", src_ip, dst_ip);
            }
            etherparse::NetHeaders::Ipv6(ip6, _) => {
                let src_ip = std::net::Ipv6Addr::from(ip6.source);
                let dst_ip = std::net::Ipv6Addr::from(ip6.destination);
                println!("{} -> {}", src_ip, dst_ip);
            }
            etherparse::NetHeaders::Arp(arp_packet) => {},
        }
    }

    Ok(())
}

pub async fn run_using_windivert_crate() -> Result<()> {
    info!("Running example using windivert crate");

    let port = 53124;
    setup_client_server(port)?;
    // let filter = format!("ip && tcp && tcp.DstPort == {}", port);
    // let filter = format!("ip && tcp && tcp.SrcPort == {}", port);
    let filter = format!("ip && tcp && loopback && (tcp.SrcPort == {} || tcp.DstPort == {})", port, port);

    info!("Windivert filter: \"{filter}\"");
    let priority = 0;
    let flags = windivert::prelude::WinDivertFlags::new().set_sniff();
    let windivert = windivert::WinDivert::network(filter, priority, flags)?;

    loop {
        let mut buffer = [0u8; 65535];
        let result = windivert.recv(Some(&mut buffer))?;
        on_receive(result)?;
    }
    
    Ok(())
}

pub fn on_receive(result: windivert::prelude::WinDivertPacket<'_, windivert::prelude::NetworkLayer>) -> Result<()> {
    let sliced = SlicedPacket::from_ip(&result.data)?;
    let headers = PacketHeaders::from_ip_slice(&result.data)?;
    
    if let Some(net) = headers.net {
        match net {
            etherparse::NetHeaders::Ipv4(ip4, _) => {
                let src_ip = std::net::Ipv4Addr::from(ip4.source);
                let dst_ip = std::net::Ipv4Addr::from(ip4.destination);
                println!("{} -> {}", src_ip, dst_ip);
            }
            etherparse::NetHeaders::Ipv6(ip6, _) => {
                let src_ip = std::net::Ipv6Addr::from(ip6.source);
                let dst_ip = std::net::Ipv6Addr::from(ip6.destination);
                println!("{} -> {}", src_ip, dst_ip);
            }
            etherparse::NetHeaders::Arp(arp_packet) => {},
        }
    }

    Ok(())
}