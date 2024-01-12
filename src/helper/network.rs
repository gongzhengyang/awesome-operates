use std::net::{IpAddr, Ipv4Addr};

pub const INTERFACE_VIRTUAL_DIR: &str = "/sys/devices/virtual/net";

/// fetch virtual network interfaces
/// resp will change when network changed
pub async fn get_virtual_interfaces() -> anyhow::Result<Vec<String>> {
    let mut interfaces = vec![];
    let mut entries = tokio::fs::read_dir(INTERFACE_VIRTUAL_DIR).await?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        interfaces.push(entry.file_name().into_string().unwrap_or_default());
    }
    Ok(interfaces)
}

pub fn sync_get_virtual_interfaces() -> anyhow::Result<Vec<String>> {
    let mut interfaces = vec![];
    let mut entries = std::fs::read_dir(INTERFACE_VIRTUAL_DIR)?;
    while let Some(Ok(entry)) = entries.next() {
        interfaces.push(entry.file_name().into_string().unwrap_or_default());
    }
    Ok(interfaces)
}

#[cfg(unix)]
pub fn get_interface_ips() -> Vec<Ipv4Addr> {
    let mut results = vec![];
    for interface in pnet_datalink::interfaces() {
        if let Some(ip) = interface
            .ips
            .iter()
            .find(|ip| ip.is_ipv4())
            .map(|ip| match ip.ip() {
                IpAddr::V4(ip) => ip,
                _ => unreachable!(),
            })
        {
            results.push(ip);
        }
    }
    results
}

#[cfg(windows)]
pub fn get_interface_ips() -> Vec<Ipv4Addr> {
    let mut results = vec![];
    for adapter in ipconfig::get_adapters().unwrap_or(vec![]) {
        results.extend(
            adapter
                .ip_addresses()
                .iter()
                .map(|x| {
                    if let IpAddr::V4(y) = x {
                        Some(y.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Option<Ipv4Addr>>>(),
        );
    }
    results
        .into_iter()
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect::<Vec<Ipv4Addr>>()
}
