use encoding::Encoding;

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
    String::from_utf8(bytes).unwrap_or_else(|e| {
        encoding::all::GBK
            .decode(e.as_bytes(), encoding::DecoderTrap::Ignore)
            .unwrap_or_else(|e| {
                tracing::error!("GBK decode error {e:?}");
                "".to_owned()
            })
    })
}

pub fn stop_guard_before_extract() {
    std::process::Command::new("taskkill.exe")
        .args(["/F", "/T", "/IM", "BoleanGuardApp.exe"])
        .output()
        .unwrap();

    std::process::Command::new("net")
        .args(["stop", "bl_guard_fs"])
        .output()
        .unwrap();
}
