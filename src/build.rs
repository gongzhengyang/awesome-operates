/// create a `build.rs` at the same folder with `Cargo.toml`
/// ```
/// //build.rs
///# fn main() {
///#     awesome_operates::build::build_init();
///# }
/// ```
/// then, you can use
/// ```
/// //pub fn about() -> &'static str {
/// //   concat!(
/// //   "\nversion: ",
/// //   env!("CARGO_PKG_VERSION"),
/// //   "\nbranch: ",
/// //   env!("GIT_BRANCH"),
/// //   "\ncommit: ",
/// //   env!("GIT_COMMIT"),
/// //   "\ngit dirty: ",
/// //   env!("GIT_DIRTY"),
/// //   "\nbuild datetime: ",
/// //   env!("BUILD_DATETIME")
/// //   )
/// // }
/// ```
pub fn build_init() {
    println!(
        "cargo:rustc-env=GIT_COMMIT={}",
        build_data::get_git_commit().unwrap()
    );
    println!(
        "cargo:rustc-env=GIT_BRANCH={}",
        build_data::get_git_branch().unwrap()
    );
    println!(
        "cargo:rustc-env=GIT_DIRTY={}",
        build_data::get_git_dirty().unwrap()
    );
    println!("cargo:rustc-env=BUILD_DATETIME={}", chrono::Local::now());
}
