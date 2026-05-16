use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy_bidirectional};
use anyhow::{Result, Context};

#[allow(dead_code)]
pub struct Transport {
    pub or_port: Option<SocketAddr>,
}

impl Transport {
#[allow(dead_code)]
    pub fn new(or_port: Option<SocketAddr>) -> Self {
        Self { or_port }
    }

    pub async fn listen(&self, addr: SocketAddr) -> Result<TcpListener> {
        let listener = TcpListener::bind(addr).await?;
        Ok(listener)
    }

    pub async fn handle_conn(&self, mut conn: TcpStream) -> Result<()> {
        let or_addr = self.or_port.context("ORPort not configured")?;
        let mut or_conn = TcpStream::connect(or_addr).await.context("Failed to connect to ORPort")?;

        copy_bidirectional(&mut conn, &mut or_conn).await?;
        Ok(())
    }
}
