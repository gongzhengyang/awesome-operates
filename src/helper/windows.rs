pub async fn is_pid_running(pid: &u32) -> bool {
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
