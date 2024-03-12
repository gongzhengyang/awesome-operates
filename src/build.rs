/// create a `build.rs` at the same folder with `Cargo.toml`
/// ```
/// //build.rs
///# fn main() {
///#     awesome_operates::build::build_init();
///# }
/// ```
/// then, you can use
///```
///  use awesome_operates::build::program_about;
///  fn main() {
///     println!("{}", program_about());
/// }
/// ```
#[inline]
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

pub fn program_about() -> &'static str {
    let values = [
        ("version", option_env!("CARGO_PKG_VERSION")),
        ("branch", option_env!("GIT_BRANCH")),
        ("commit", option_env!("GIT_COMMIT")),
        ("git dirty", option_env!("GIT_DIRTY")),
        ("build datetime", option_env!("BUILD_DATETIME"))
    ].iter().map(|(k, v)| format!("{k}: {}", v.unwrap_or_default())).collect::<Vec<String>>().join("\n");
    let value = Box::new(values);
    Box::leak(value)
}
