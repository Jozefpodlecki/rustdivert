
use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use etherparse::{PacketHeaders, SlicedPacket};
use flexi_logger::Logger;
use rustdivert::{WinDivertFlags, sync::Windivert};
use tokio::{signal, sync::Mutex, time::sleep};
use crate::{client::Client, server::Server};
use log::*;

mod server;
mod client;
mod examples;

async fn run_using_rustdivert() -> Result<()> {
    info!("Running example using rustdivert");

    let mut server = Server::new();
    let port = 53124;
    let addr = "127.0.0.1:53124";
    server.start(addr)?;

    let filter = format!("ip && tcp");
    info!("Windivert filter: \"{filter}\"");
    
    let priority = 0;
    let flags = WinDivertFlags::new().set_sniff();
    let windivert = Arc::new(Mutex::new(Some(Windivert::open(
        rustdivert::WinDivertLayer::Network, 
        &filter, 
        priority, 
        flags
    )?)));

    let windivert_clone = windivert.clone();
    
    // Handle Ctrl+C
    let ctrl_c_task = tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down...");
                // Close WinDivert handle
                let mut guard = windivert_clone.lock().await;
                if let Some(w) = guard.take() {
                    drop(w);
                    info!("WinDivert handle closed");
                }
                std::process::exit(0);
            }
            Err(e) => {
                error!("Unable to listen for shutdown signal: {}", e);
            }
        }
    });

    // Main packet processing loop
    let processing_task = tokio::spawn(async move {
        loop {
            let mut buffer = [0u8; 65535];
            info!("Receiving");
            
            let result = {
                let guard = windivert.lock().await;
                if let Some(w) = guard.as_ref() {
                    w.recv(&mut buffer)
                } else {
                    info!("WinDivert closed, exiting loop");
                    break;
                }
            };
            
            match result {
                Ok(_) => info!("Received"),
                Err(e) => {
                    error!("Receive error: {}", e);
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    tokio::select! {
        _ = ctrl_c_task => {},
        _ = processing_task => {},
    }

    Ok(())
}

async fn run_using_windivert_crate() -> Result<()> {
    info!("Running example using windivert crate");

    let mut server = Server::new();

    let port = 53124;
    let addr = "127.0.0.1:53124";
    server.start(addr)?;

    let mut client = Client::new();
    client.start(addr);

    // let filter = format!("ip && tcp && tcp.DstPort == {}", port);
    // let filter = format!("ip && tcp && tcp.SrcPort == {}", port);
    let filter = format!("ip && tcp && loopback && (tcp.SrcPort == {} || tcp.DstPort == {})", port, port);
    // let filter = "ip && tcp && ip.SrcAddr == 127.0.0.1 && ip.DstAddr == 127.0.0.1";
//     let filter = format!(
//     "ip && tcp && (tcp.SrcPort == {} || tcp.DstPort == {}) && ip.SrcAddr == 127.0.0.1 && ip.DstAddr == 127.0.0.1",
//     port, port
// );
    info!("Windivert filter: \"{filter}\"");
    let priority = 0;
    let flags = windivert::prelude::WinDivertFlags::new().set_sniff();
    let windivert = windivert::WinDivert::network(filter, priority, flags)?;
    // 2147942487 decimal = 0x80070057
    // windivert.close(action);

    loop {
        let mut buffer = [0u8; 65535];
        info!("Receiving");
        let result = windivert.recv(Some(&mut buffer))?;
        info!("Received");
        // dbg!(result.data);

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
        
        sleep(Duration::from_secs(1)).await;
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    Logger::try_with_str(
        "debug,rustdivert_examples::client=error,rustdivert_examples::server=error"
    )?.start()?;

    run_using_rustdivert().await.unwrap();
    // run_using_windivert_crate().await.unwrap();
    // if let Err(err) = run().await {
    //     error!("{err}");
    //     if let Some(backtrace) = err.backtrace().status().into() {
    //         error!("Backtrace:\n{:?}", backtrace);
    //     }
    // }

    Ok(())
}