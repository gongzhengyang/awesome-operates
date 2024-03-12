use serde::Serialize;
use tokio::process::Command;

pub async fn is_pid_running<T>(pid: T) -> bool
where
    T: Serialize,
{
    let pid = serde_json::json!(pid).as_u64().unwrap_or(0);
    if pid.eq(&0) {
        return false;
    }
    let output = Command::new("powershell")
        .args(&["Get-Process", "-id", &pid.to_string()])
        .output()
        .await
        .unwrap();
    let output = String::from_utf8_lossy(&output.stdout);
    output.contains("ProcessName") && output.contains("Id")
}

pub fn show_bytes(bytes: Vec<u8>) -> String {
    encoding_rs::GBK.decode(&bytes).0.to_string()
}

pub fn set_execute_permission(_filepath: &str) {}
