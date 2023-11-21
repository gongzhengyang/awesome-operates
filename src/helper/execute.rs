use std::process::{Output, Stdio};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

pub async fn execute_command(cmd: &str) -> anyhow::Result<Output> {
    tracing::info!("execute command `{cmd}`");
    Ok(Command::new("sh").args(["-c", cmd]).output().await?)
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
