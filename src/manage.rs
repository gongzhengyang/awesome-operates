use std::path::Path;
use std::process::Output;

use crate::helper::{self, get_program_args};

pub async fn register_service(
    service_name: &str,
    exclude_args: &Vec<&str>,
    restart: bool,
) -> anyhow::Result<Output> {
    let exe_filepath = std::env::current_exe()?;
    let work_directory = std::env::current_dir()?;
    let service_config = format!(
        r#"[Unit]
Description=sys layer agent service
After=network.target

[Service]
WorkingDirectory={}
ExecStart={} {}
Restart=always

[Install]
WantedBy=multi-user.target
"#,
        work_directory.display(),
        exe_filepath.display(),
        get_program_args(exclude_args).join(" ")
    );
    tokio::fs::write(service_config_path(service_name), service_config).await?;
    let mut command = format!("systemctl enable {service_name} && systemctl daemon-reload");
    if restart {
        command.push_str(&format!("&& systemctl restart {service_name}"));
    }
    helper::execute_command(&command).await
}

pub async fn reset(service_name: &str) -> anyhow::Result<Output> {
    let command = format!(
        "systemctl stop {service_name} \
        && systemctl disable {service_name} \
        && rm -f {} \
        && systemctl daemon-reload",
        service_config_path(service_name)
    );
    helper::execute_command(&command).await
}

#[inline]
pub fn service_config_path(service_name: &str) -> String {
    format!("/lib/systemd/system/{service_name}.service")
}

pub async fn check_update_binary(
    update_filepath: &str,
    original_filepath: &str,
) -> anyhow::Result<()> {
    if binary_filepath_execute_success(update_filepath)
            .await
            .unwrap_or_default()
    {
        tokio::fs::rename(update_filepath, original_filepath).await?;
    }
    Ok(())
}

pub async fn binary_filepath_execute_success(filepath: &str) -> anyhow::Result<bool> {
    if !Path::new(filepath).exists() {
        return Ok(false);
    }
    #[cfg(unix)]
    helper::execute_command(&format!("chmod a+x {}", filepath)).await?;
    Ok(helper::execute_command(filepath).await?.status.success())
}
