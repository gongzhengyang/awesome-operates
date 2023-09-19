use std::path::Path;
use std::process::ExitStatus;

use crate::helper::{self, get_program_args};

pub async fn register_agent_service(
    service_name: &str,
    exclude_args: &Vec<&str>,
    restart: bool,
) -> anyhow::Result<ExitStatus> {
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
    tokio::fs::write(
        format!("/lib/systemd/system/{service_name}.service"),
        service_config,
    )
    .await?;
    let mut command = format!("systemctl enable {service_name} && systemctl daemon-reload");
    if restart {
        command.push_str(&format!("&& systemctl restart {service_name}"));
    }
    Ok(tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .await?)
}

pub async fn check_update_binary(
    update_filepath: &str,
    original_filepath: &str,
) -> anyhow::Result<()> {
    if Path::new(update_filepath).exists() {
        helper::execute_command(&format!("chmod a+x {}", update_filepath)).await?;
        if helper::execute_command(update_filepath)
            .await?
            .status
            .success()
        {
            tokio::fs::rename(update_filepath, original_filepath).await?;
        }
    }
    Ok(())
}
