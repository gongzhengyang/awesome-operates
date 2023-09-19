use std::process::{Output, Stdio};

use regex::{Captures, Regex};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::consts;

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

pub fn default_formatted_now() -> String {
    formatted_now(consts::DEFAULT_TIME_FORMAT)
}

pub fn formatted_now(fmt: &str) -> String {
    chrono::Local::now().format(fmt).to_string()
}

#[inline]
pub fn human_bytes(mut value: f64) -> String {
    if value < 1024.0 {
        return format!("{value:.2} B");
    }
    for symbol in ["KB", "MB", "GB", "TB"] {
        value /= 1024.0;
        if value < 1024.0 {
            return format!("{value:.2} {symbol}");
        }
    }
    format!("{value}")
}

#[inline]
pub fn format_as_gb(value: u64) -> String {
    let gb = value as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{gb:.2} GB")
}

#[inline]
pub fn decimal_with_two(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default().round_dp(2)
}

#[inline]
pub fn decimal_with_four(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default().round_dp(4)
}

pub fn get_program_args(excludes: &Vec<&str>) -> Vec<String> {
    let mut args = std::env::args().collect::<Vec<String>>();
    args.drain(1..)
        .filter(|x| {
            let mut filter = true;
            for exclude in excludes {
                if x.contains(exclude) {
                    filter = false;
                    break;
                }
            }
            filter
        })
        .collect::<Vec<String>>()
}

pub fn get_pkg_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn calculate_agent_version(version: &str) -> u32 {
    let re = Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)").unwrap();
    let value = |x: &Captures, field: &str| x.name(field).unwrap().as_str().parse::<u8>().unwrap();
    let (major, minor, patch) = re.captures(version).map_or((0, 0, 0), |c| {
        (value(&c, "major"), value(&c, "minor"), value(&c, "patch"))
    });
    (major as u32) << 16 | (minor as u32) << 8 | patch as u32
}

pub async fn add_execute_permission(filepath: &str) -> anyhow::Result<()> {
    #[cfg(unix)]
    execute_command(&format!("chmod a+x {}", filepath)).await?;
    Ok(())
}
