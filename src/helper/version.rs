use regex::{Captures, Regex};
use snafu::{IntoError, NoneError, ResultExt};
use tokio::process::Command;

use crate::error::{BinaryCannotBeExecuteSnafu, CommonIoSnafu, Result};

use super::show_bytes;

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

pub async fn get_binary_file_version(exe_filepath: &str) -> Result<String> {
    let output = Command::new(exe_filepath)
        .arg("--version")
        .output()
        .await
        .context(CommonIoSnafu)?;
    if !output.status.success() {
        return Err(BinaryCannotBeExecuteSnafu {
            filepath: exe_filepath.to_owned(),
        }
        .into_error(NoneError));
    }
    Ok(show_bytes(output.stdout))
}
