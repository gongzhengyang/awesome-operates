use regex::{Captures, Regex};

pub use execute::{execute_command, execute_command_with_args_sender};
pub use format::{
    decimal_with_four, decimal_with_two, default_formatted_now, formatted_now, human_bytes,
};
pub use iter::iter_object;
pub use network::{
    get_virtual_interfaces, sync_get_virtual_interfaces,
};

mod execute;
mod format;
mod iter;
mod network;

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
