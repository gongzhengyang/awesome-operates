use snafu::ResultExt;
use std::path::Path;
use std::process::Output;

use crate::error::{CommonIoSnafu, Result};
use crate::helper::{self, get_program_args};

/// register current program with command args as a service and enable it
pub async fn register_service(
    service_name: &str,
    exclude_args: &Vec<&str>,
    restart: bool,
) -> Result<Output> {
    let exe_filepath = std::env::current_exe().context(CommonIoSnafu)?;
    let work_directory = std::env::current_dir().context(CommonIoSnafu)?;
    let service_config = format!(
        r#"[Unit]
Description= agent service
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
    tokio::fs::write(service_config_path(service_name), service_config)
        .await
        .context(CommonIoSnafu)?;
    let mut command = format!("systemctl daemon-reload && systemctl enable {service_name}");
    if restart {
        command.push_str(&format!("&& systemctl restart {service_name}"));
    }
    helper::execute_command(&command).await
}

///reset service
/// stop the service
/// disable the service
pub async fn reset(service_name: &str) -> Result<Output> {
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

pub async fn check_update_binary(update_filepath: &str, original_filepath: &str) -> Result<()> {
    tracing::debug!("check update binary");
    if binary_filepath_execute_success(update_filepath)
        .await
        .unwrap_or_default()
    {
        tracing::info!("rename {update_filepath} into {original_filepath}");
        tokio::fs::rename(update_filepath, original_filepath)
            .await
            .context(CommonIoSnafu)?;
    }
    Ok(())
}

/// check filepath binary can execute success
pub async fn binary_filepath_execute_success(filepath: &str) -> Result<bool> {
    tracing::debug!("check binary execute {filepath}");
    if !Path::new(filepath).exists() {
        return Ok(false);
    }
    #[cfg(unix)]
    helper::add_execute_permission(filepath).await?;
    Ok(helper::execute_command(&format!("{filepath} --version"))
        .await?
        .status
        .success())
}
