use cfg_if::cfg_if;

pub use execute::{execute_command, execute_command_with_args_sender, kill_process_by_pid};
pub use format::{
    decimal_with_four, decimal_with_two, default_formatted_now, format_from_timestamp,
    formatted_now, human_bytes,
};
pub use iter::iter_object;
pub use network::{get_virtual_interfaces, sync_get_virtual_interfaces};
pub use version::{calculate_agent_version, get_binary_file_version, get_pkg_version};

mod execute;
mod format;
mod iter;
mod network;
mod version;

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
pub async fn add_execute_permission(filepath: &str) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = tokio::fs::metadata(filepath).await?.permissions();
    permissions.set_mode(0o777);
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
