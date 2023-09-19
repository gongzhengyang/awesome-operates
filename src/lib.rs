pub mod build;
pub mod consts;
pub mod graceful;
pub mod helper;
pub mod manage;
pub mod schedule;

#[macro_export]
macro_rules! extract_all_files {
    ($asset:ident) => {
        for file in $asset::iter() {
            tracing::info!("extract {}", file.as_ref());
            let filepath = file.as_ref();
            if let Some(parent) = std::path::Path::new(filepath).parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }
            let file = $asset::get(filepath).unwrap().data;
            tokio::fs::write(filepath, file).await?;
            $crate::helper::add_execute_permission(filepath).await?;
        }
    };
}
