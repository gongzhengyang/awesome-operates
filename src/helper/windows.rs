use serde::Serialize;

pub async fn is_pid_running<T>(pid: T) -> bool
where
    T: Serialize,
{
    let pid = serde_json::json!(pid).as_u64().unwrap_or(0);
    let query = format!("PID eq {}", pid);
    let child = tokio::process::Command::new("cmd")
        .args([r"/C", "tasklist", r"/FI", query.as_str()])
        .output()
        .await
        .unwrap();

    let stdout = show_bytes(child.stdout);
    stdout.contains(format!("{pid}").as_str())
}

pub fn show_bytes(bytes: Vec<u8>) -> String {
    encoding_rs::GBK.decode(&bytes).0.to_string()
}

pub fn set_execute_permission(_filepath: &str) {}
