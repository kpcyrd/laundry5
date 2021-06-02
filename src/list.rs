use crate::errors::*;
use std::net::SocketAddr;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

pub async fn load(path: &Path) -> Result<Vec<SocketAddr>> {
    let f = File::open(path)
        .await
        .with_context(|| anyhow!("Failed to open file: {:?}", path))?;
    let f = BufReader::new(f);

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

    debug!("Loaded {} proxies", proxies.len());

    Ok(proxies)
}
