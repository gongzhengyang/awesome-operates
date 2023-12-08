use serde::Serialize;

pub async fn is_pid_running<T>(pid: T) -> bool
where
    T: Serialize,
{
    let value = serde_json::json!(pid);
    if value.as_u64().is_some() {
        std::path::Path::new(format!("/proc/{value}").as_str()).exists()
    } else {
        false
    }
}

pub fn set_execute_permission(filepath: &str) {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o777);
    std::fs::set_permissions(filepath, perms).unwrap();
}

pub fn show_bytes(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap()
}
