use std::{net::SocketAddr, time::Duration};

use log::*;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, ToSocketAddrs}, spawn, task::spawn_blocking, time::sleep};
use anyhow::Result;

pub struct Server {
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start<A: tokio::net::ToSocketAddrs + Send + std::fmt::Debug + 'static>(&mut self, addr: A) -> Result<()> {
        let handle = spawn(async move {
            if let Err(e) = Self::start_inner(addr).await {
                error!("Server failed: {:?}", e);
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    async fn start_inner<A: tokio::net::ToSocketAddrs + std::fmt::Debug>(addr: A) -> Result<()> {
        info!("Launching server at {:?}", addr);
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (mut socket, client_addr) = listener.accept().await?;
            info!("Accepted connection from {}", client_addr);

            Self::send_messages(socket, client_addr);
        }
    }

    fn send_messages(mut socket: TcpStream, client_addr: SocketAddr) {
        spawn(async move {
            loop {
                let msg = b"1234";
                if let Err(e) = socket.write_all(msg).await {
                    error!("Failed to send to {}: {:?}", client_addr, e);
                    break;
                }

                sleep(Duration::from_secs(1)).await;
            }

            info!("Finished sending to {}", client_addr);
        });
    }
}