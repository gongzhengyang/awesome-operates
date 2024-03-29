use std::path::{Path, PathBuf};

use cfg_if::cfg_if;
use snafu::ResultExt;

pub use execute::{
    add_execute_permission, execute_command, execute_command_with_args_sender,
    is_current_running_newer, kill_process_by_pid, remove_file_when_older,
    sync_add_execute_permission,
};
pub use format::{
    decimal_with_four, decimal_with_two, default_formatted_now, format_from_timestamp,
    formatted_now, human_bytes,
};
pub use fs::create_file_parent_dir;
pub use iter::iter_object;
pub use network::{get_interface_ips, get_virtual_interfaces, sync_get_virtual_interfaces};
pub use version::{calculate_agent_version, get_binary_file_version, get_pkg_version};

use crate::error::{CommonIoSnafu, Result, ZipExtractSnafu};

mod execute;
mod format;
mod iter;
mod network;

mod fs;
mod version;

cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else {
        mod unix;
        pub use unix::*;
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

/// one can
pub fn write_filepath_with_data(filepath: impl AsRef<Path>, file: impl AsRef<[u8]>) -> Result<()> {
    if let Some(parent) = filepath.as_ref().parent() {
        std::fs::create_dir_all(parent).context(CommonIoSnafu)?;
    }
    if std::fs::write(&filepath, &file)
        .context(CommonIoSnafu)
        .is_err()
    {
        #[cfg(unix)]
        try_rewrite(&filepath, &file)?;
    }
    if filepath.as_ref().extension().is_some_and(|v| v.eq("zip")) {
        zip_extensions::zip_extract(&filepath.as_ref().to_path_buf(), &PathBuf::new())
            .context(ZipExtractSnafu)?;
    }
    cfg_if! {
        if #[cfg(unix)] {
            use snafu::OptionExt;

            sync_add_execute_permission(filepath.as_ref().to_str().context(crate::error::OptionNoneSnafu)?)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn try_rewrite(filepath: impl AsRef<Path>, file: impl AsRef<[u8]>) -> Result<()> {
    use std::time::Duration;
    let path = filepath.as_ref().to_owned();
    tokio::spawn(async move {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        if !filename.is_empty() {
            let _ = execute_command(&format!("pkill -9 {}", filename)).await;
        }
    });
    std::thread::sleep(Duration::from_secs(5));
    std::fs::write(&filepath, file).context(CommonIoSnafu)?;
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
