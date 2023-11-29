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
