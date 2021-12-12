use crate::errors::*;
use std::net::SocketAddr;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};

pub async fn load_from_path(path: &Path) -> Result<Vec<SocketAddr>> {
    let f = File::open(path)
        .await
        .with_context(|| anyhow!("Failed to open file: {:?}", path))?;
    let f = BufReader::new(f);
    load_from_reader(f).await
}

async fn load_from_reader<T: AsyncBufRead + Unpin>(f: T) -> Result<Vec<SocketAddr>> {
    let mut proxies = Vec::new();

    let mut lines = f.lines();
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let proxy = line
            .parse::<SocketAddr>()
            .with_context(|| anyhow!("Invalid proxy in list: {:?}", line))?;

        proxies.push(proxy);
    }

    info!("Loaded {} proxies from file", proxies.len());

    Ok(proxies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[tokio::test]
    async fn test_parse_empty() {
        let buf = b"";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(&list, &[]);
    }

    #[tokio::test]
    async fn test_parse_empty_newline() {
        let buf = b"\n";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(&list, &[]);
    }

    #[tokio::test]
    async fn test_parse_empty_two_newlines() {
        let buf = b"\n\n";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(&list, &[]);
    }

    #[tokio::test]
    async fn test_parse_empty_comment() {
        let buf = b"# this is a comment";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(&list, &[]);
    }

    #[tokio::test]
    async fn test_parse_ipv4() {
        let buf = b"192.0.2.1:1337";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(
            &list,
            &[SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
                1337
            )]
        );
    }

    #[tokio::test]
    async fn test_parse_ipv6() {
        let buf = b"[2001:0DB8::12:34]:1337";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(
            &list,
            &[SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0x12, 0x34)),
                1337
            )]
        );
    }

    #[tokio::test]
    async fn test_parse_mixed() {
        let buf = b"192.0.2.1:1337
192.0.2.2:1337
192.0.2.3:1337
[2001:0DB8::12:34]:1337
[2001:0DB8::56:78]:1337


";
        let reader = BufReader::new(Cursor::new(buf));
        let list = load_from_reader(reader).await.unwrap();
        assert_eq!(
            &list,
            &[
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 1337),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)), 1337),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)), 1337),
                SocketAddr::new(
                    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0x12, 0x34)),
                    1337
                ),
                SocketAddr::new(
                    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0x56, 0x78)),
                    1337
                ),
            ]
        );
    }
}
