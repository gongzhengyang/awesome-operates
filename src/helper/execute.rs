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
        .args(args)
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
