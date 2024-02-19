use std::fmt::Display;
use std::path::Path;
use std::process::{Output, Stdio};

use cfg_if::cfg_if;
use serde::Serialize;
use snafu::ResultExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::error::{CommonIoSnafu, Result};

pub async fn execute_command(cmd: &str) -> Result<Output> {
    tracing::info!("execute command `{cmd}`");
    Command::new("sh")
        .args(["-c", cmd])
        .output()
        .await
        .context(CommonIoSnafu)
}

pub async fn execute_command_with_args_sender(cmd: &str, args: Vec<String>, tx: Sender<String>) {
    tracing::info!("execute command `{cmd}` with args: `{args:?}`");
    let mut child = Command::new(cmd)
        .kill_on_drop(true)
        .args(args.clone())
        .stdout(Stdio::piped())
        .spawn()
        .expect("fail to execute");
    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let mut reader = BufReader::new(stdout).lines();

    tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .expect("child process encountered an error");

        tracing::info!("child status was: {status}");
    });

    while let Some(line) = reader
        .next_line()
        .await
        .unwrap_or(Some("fail to get output".to_owned()))
    {
        tx.send(line).await.unwrap();
    }
    tracing::debug!("cmd:[{cmd}] with args {args:?} is over");
}

/// ```rust
/// use awesome_operates::helper::kill_process_by_pid;
/// # async {
/// #     kill_process_by_pid(999990).await;
/// #     kill_process_by_pid(Some(99999999)).await;
/// #     // this will has no use
/// #     kill_process_by_pid(None::<u32>).await;
/// # };
/// ```
pub async fn kill_process_by_pid<T>(pid: T)
where
    T: Serialize,
{
    let value = serde_json::json!(pid);
    if value.as_u64().is_none() {
        return;
    }
    let pid = value.to_string();
    let pid_str = pid.as_str();
    cfg_if! {
        if #[cfg(unix)] {
           let status = tokio::process::Command::new("kill")
            .args(["-9", pid_str])
            .status()
            .await
            .unwrap();
        } else {
            let status = tokio::process::Command::new("taskkill")
                .args([r"/T", r"/F", r"/PID", pid_str])
                .status()
                .await
                .unwrap();
        }
    }
    tracing::info!("kill process {pid_str} with exit status {status:?}");
}

pub async fn remove_file_when_older(filepath: impl AsRef<Path> + Display) {
    inner_remove_file_when_older(&filepath)
        .await
        .unwrap_or_default()
}

async fn inner_remove_file_when_older(filepath: impl AsRef<Path> + Display) -> std::io::Result<()> {
    if is_current_running_newer(&filepath).unwrap_or(false) {
        tracing::info!("remove {filepath} because it`s older");
        tokio::fs::remove_file(filepath).await?;
    }
    Ok(())
}

pub fn is_current_running_newer(filepath: impl AsRef<Path> + Display) -> std::io::Result<bool> {
    let check = std::fs::metadata(&filepath)?.modified()?;
    let current_running = std::fs::metadata(std::env::current_exe()?)?.modified()?;
    // let check = tokio::fs::metadata(&filepath).await?.modified()?;
    // let current_running = tokio::fs::metadata(std::env::current_exe()?)
    //     .await?
    //     .modified()?;
    let newer = current_running > check;
    let format_time = |v| chrono::DateTime::<chrono::offset::Local>::from(v);
    tracing::debug!(
        "check file modified current: {:?} check [{filepath}]{:?} newer: {newer}",
        format_time(current_running),
        format_time(check)
    );
    Ok(newer)
}
