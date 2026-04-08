use std::time::Duration;

use anyhow::{Result, anyhow};
use etherparse::{PacketHeaders, SlicedPacket};
use flexi_logger::Logger;
use rustdivert::{WinDivertFlags, Windivert};
use tokio::time::sleep;
use crate::{client::Client, server::Server};
use log::*;

mod server;
mod client;
mod examples;

async fn run_using_rustdivert() -> Result<()> {
    info!("Running example using rustdivert");

    // $env:RUST_BACKTRACE=1
    // Start-Process powershell -Verb runAs -ArgumentList "-NoExit", "-Command", "Set-Location '$pwd'; `$env:RUST_BACKTRACE='1'; cargo run"

    let mut server = Server::new();

    let port = 53124;
    let addr = "127.0.0.1:53124";
    server.start(addr)?;

    let layer = rustdivert::WinDivertLayer::Network;
    // let filter = format!("ip && tcp && tcp.DstPort == {}", port);
    let filter = format!("ip && tcp");
    // let filter = format!("ip && tcp && loopback && (tcp.SrcPort == {} || tcp.DstPort == {})", port, port);
    
    info!("Windivert filter: \"{filter}\"");
    let priority = 0;
    let flags = WinDivertFlags::new().set_sniff();
    let windivert = Windivert::open(layer, &filter, priority, flags)?;

    loop {
        let mut buffer = [0u8; 65535];
        info!("Receiving");
        let result = windivert.recv(&mut buffer)?;
        info!("Received");
        sleep(Duration::from_secs(1)).await;
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