use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use cfg_if::cfg_if;
use snafu::{OptionExt, ResultExt};

pub use execute::{execute_command, execute_command_with_args_sender, kill_process_by_pid, remove_file_when_older};
pub use format::{
    decimal_with_four, decimal_with_two, default_formatted_now, format_from_timestamp,
    formatted_now, human_bytes,
};
pub use fs::create_file_parent_dir;
pub use iter::iter_object;
pub use network::{get_virtual_interfaces, sync_get_virtual_interfaces, get_interface_ips};
pub use version::{calculate_agent_version, get_binary_file_version, get_pkg_version};

use crate::error::{CommonIoSnafu, OptionNoneSnafu, Result, ZipExtractSnafu};

mod execute;
mod format;
mod iter;
mod network;

mod version;
mod fs;

cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
    } else {
        mod windows;
        pub use windows::*;
    }
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

#[cfg(unix)]
pub async fn add_execute_permission(filepath: &str) -> Result<Output> {
    let command = format!("chmod a+x {}", filepath);
    execute_command(&command).await
}

#[cfg(unix)]
pub fn sync_add_execute_permission(filepath: &str) -> Result<Output> {
    let command = format!("chmod a+x {}", filepath);
    std::process::Command::new("sh")
        .args(["-c", &command])
        .output().context(CommonIoSnafu)
}

/// one can
pub fn write_filepath_with_data(
    filepath: impl AsRef<Path>,
    file: impl AsRef<[u8]>,
) -> Result<()> {
    if let Some(parent) = filepath.as_ref().parent() {
        std::fs::create_dir_all(parent)
            .context(CommonIoSnafu)?;
    }
    if std::fs::write(&filepath, &file).context(CommonIoSnafu).is_err() {
        let path = filepath.as_ref().to_owned();
        #[cfg(unix)]
        tokio::spawn(async move {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if !filename.is_empty() {
                let _ = execute_command(&format!("pkill -9 {}", filename)).await;
            }
        });
        std::thread::sleep(Duration::from_secs(5));
        std::fs::write(&filepath, file).context(CommonIoSnafu)?;
    }
    if filepath.as_ref().extension().is_some_and(|v| v.eq("zip")) {
        zip_extensions::zip_extract(&filepath.as_ref().to_path_buf(), &PathBuf::new())
            .context(ZipExtractSnafu)?;
    }
    #[cfg(unix)]
    sync_add_execute_permission(filepath.as_ref().to_str().context(OptionNoneSnafu)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_version() {
        assert_eq!(
            calculate_agent_version("agent-10.22.33.exe"),
            10 << 16 | 22 << 8 | 33
        );
        assert_eq!(calculate_agent_version("agent-1.2.3"), 1 << 16 | 2 << 8 | 3);
    }
}
