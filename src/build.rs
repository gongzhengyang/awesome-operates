/// create a `build.rs` at the same folder with `Cargo.toml`
/// ```
/// //build.rs
///# fn main() {
///     awesome_operates::build::build_init();
///# }
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
