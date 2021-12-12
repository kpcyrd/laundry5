use crate::errors::*;
use bstr::ByteSlice;
use rand::prelude::SliceRandom;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub enum SocksAddr<'a> {
    Ipv4(&'a [u8]),
    Domain(&'a [u8]),
    Ipv6(&'a [u8]),
}

impl<'a> fmt::Display for SocksAddr<'a> {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SocksAddr::Ipv4(b) => write!(w, "{}.{}.{}.{}", b[0], b[1], b[2], b[3]),
            SocksAddr::Domain(b) => write!(w, "{:?}", b.as_bstr()),
            SocksAddr::Ipv6(b) => write!(w, "[{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}:{:02X?}{:02X?}]",
                b[0], b[1], b[2], b[3],
                b[4], b[5], b[6], b[7],
                b[8], b[9], b[10], b[11],
                b[12], b[13], b[14], b[15],
            ),
        }
    }
}

pub async fn recv_handshake<'a>(
    socket: &mut TcpStream,
    addr_buf: &'a mut [u8],
) -> Result<(SocksAddr<'a>, u16)> {
    let mut buf = [0u8; 2];
    socket
        .read_exact(&mut buf)
        .await
        .context("Failed to read handshake")?;

    if buf[0] != 5 {
        bail!("Unexpected socks version");
    }

    let n = buf[1] as usize;

    if n == 0 {
        bail!("Got empty list of supported authentication methods");
    }

    let mut buf = [0u8; 255];
    socket
        .read_exact(&mut buf[..n])
        .await
        .context("Failed to read handshake")?;

    // TODO: we're going to ignore the list and expect the client to support no-auth

    socket
        .write_all(&[0x05, 0x00])
        .await
        .context("Failed to send handshake")?;

    let mut buf = [0u8; 4];
    socket
        .read_exact(&mut buf)
        .await
        .context("Failed to read handshake")?;

    if buf[0] != 5 {
        bail!("Unexpected socks version");
    }

    if buf[1] != 1 {
        bail!("Only tcp/ip stream connections are supported");
    }

    if buf[2] != 0 {
        bail!("Reserved field is not zero");
    }

    let addr = match buf[3] {
        1 => {
            let buf = &mut addr_buf[..4];
            socket
                .read_exact(buf)
                .await
                .context("Failed to read handshake")?;
            SocksAddr::Ipv4(buf)
        }
        3 => {
            let n = socket.read_u8().await.context("Failed to read handshake")? as usize;
            let buf = &mut addr_buf[..n];
            socket
                .read_exact(buf)
                .await
                .context("Failed to read handshake")?;
            SocksAddr::Domain(buf)
        }
        4 => {
            let buf = &mut addr_buf[..16];
            socket
                .read_exact(buf)
                .await
                .context("Failed to read handshake")?;
            SocksAddr::Ipv6(buf)
        }
        x => {
            bail!("Unsupported address type: {}", x);
        }
    };

    let port = socket
        .read_u16()
        .await
        .context("Failed to read handshake")?;

    Ok((addr, port))
}

async fn connect(proxy_addr: &SocketAddr, addr: &SocksAddr<'_>, port: u16) -> Result<TcpStream> {
    debug!("Connecting to proxy server at {}", proxy_addr);
    let mut proxy = TcpStream::connect(proxy_addr)
        .await
        .context("Failed to connect to proxy server")?;
    debug!("Connected to {:?}", proxy_addr);

    proxy
        .write_all(&[5, 1, 0])
        .await
        .context("Failed to send handshake")?;

    let mut buf = [0u8; 2];
    proxy
        .read_exact(&mut buf)
        .await
        .context("Failed to read handshake")?;

    if buf != [5, 0] {
        bail!("Proxy didn't accept anonymous auth");
    }

    let mut buf = vec![5, 1, 0];

    match addr {
        SocksAddr::Ipv4(b) => {
            buf.push(1);
            buf.extend(*b);
        }
        SocksAddr::Domain(b) => {
            buf.push(3);
            buf.push(b.len() as u8);
            buf.extend(*b);
        }
        SocksAddr::Ipv6(b) => {
            buf.push(4);
            buf.extend(*b);
        }
    }

    buf.extend(&port.to_be_bytes());

    proxy
        .write_all(&buf)
        .await
        .context("Failed to send handshake")?;

    Ok(proxy)
}

pub async fn serve(mut socket: TcpStream, proxies: Arc<Vec<SocketAddr>>) -> Result<()> {
    let mut buf = [0u8; 255];
    let (addr, port) = recv_handshake(&mut socket, &mut buf)
        .await
        .context("Failed to complete handshake with client")?;

    debug!("Received connection request for {}:{}", addr, port);

    let proxy = proxies
        .choose(&mut rand::thread_rng())
        .context("No proxies configured")?;
    debug!("Picked random proxy: {}", proxy);

    let mut proxy = connect(proxy, &addr, port)
        .await
        .context("Failed to complete handshake with proxy")?;

    tokio::io::copy_bidirectional(&mut socket, &mut proxy)
        .await
        .context("Failed to relay data")?;

    debug!("Connection finished");

    Ok(())
}
