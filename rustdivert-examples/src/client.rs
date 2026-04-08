use std::net::SocketAddr;
use tokio::{io::AsyncReadExt, net::{TcpStream, ToSocketAddrs}, spawn};
use log::*;

pub struct Client {
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Client {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start<A: ToSocketAddrs + Send + std::fmt::Debug + 'static>(&mut self, addr: A) {
        let handle = spawn(async move {
            match TcpStream::connect(addr).await {
                Ok(mut stream) => {
                    info!("Client connected to server");

                    let mut buf = [0u8; 1024];
                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) => {
                                info!("Server closed connection");
                                break;
                            }
                            Ok(n) => {
                                // info!("Client received {} bytes: {:?}", n, &buf[..n]);
                            }
                            Err(e) => {
                                error!("Failed to read from server: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to server: {:?}", e);
                }
            }
        });

        self.handle = Some(handle);
    }
}