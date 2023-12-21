pub mod build;
pub mod consts;
pub mod embed;
pub mod error;
pub mod graceful;
pub mod helper;
pub mod manage;
pub mod proxy;
pub mod router;
pub mod schedule;
pub mod swagger;

/// usage
/// ```
/// use rust_embed::RustEmbed;
///
/// #[derive(RustEmbed)]
/// #[folder = "src/assets"]
/// struct Asset;
///
/// async fn extract() -> anyhow::Result<()>{
/// #   awesome_operates::extract_all_files!(Asset);
///     Ok(())
/// }
///
/// ```
#[macro_export]
macro_rules! extract_all_files {
    ($asset:ty) => {
        for file in <$asset>::iter() {
            tracing::debug!("extract {}", file.as_ref());
            let filepath = file.as_ref();
            if let Some(parent) = std::path::Path::new(filepath).parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }
            let file = <$asset>::get(filepath).unwrap().data;
            tokio::fs::write(filepath, file).await?;
            #[cfg(unix)]
            $crate::helper::add_execute_permission(filepath).await?;
        }
    };
}
