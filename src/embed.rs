use async_trait::async_trait;
use rust_embed::RustEmbed;
use snafu::ResultExt;
use std::path::Path;

use crate::error::{CommonIoSnafu, Result};
use crate::helper;

/// usage
/// ```
/// use rust_embed::RustEmbed;
/// use awesome_operates::embed::AssetExtractExt;
/// use awesome_operates::error::Result;
///
/// #[derive(RustEmbed)]
/// #[folder = "src/assets"]
/// struct Asset;
///
/// impl AssetExtractExt for Asset {}
///
/// async fn extract() -> Result<()>{
///     Asset::extract().await?;
///     Ok(())
/// }
///
/// ```

#[async_trait]
pub trait AssetExtractExt: RustEmbed {
    async fn before_extract() -> Result<()> {
        Ok(())
    }

    async fn perform_extract() -> Result<()> {
        for file in Self::iter() {
            tracing::debug!("extract {}", file.as_ref());
            let filepath = file.as_ref();
            if helper::is_current_running_newer(filepath).unwrap_or(true) {
                let file = Self::get(filepath).unwrap().data.clone();
                helper::write_filepath_with_data(filepath, file)?;
            } else {
                tracing::debug!("skip {filepath} because it newer");
                continue;
            }
        }
        Ok(())
    }

    fn update_filenames() -> Vec<(String, String)> {
        vec![]
    }

    async fn update_files() -> Result<()> {
        for (src, dst) in Self::update_filenames() {
            if Path::new(&src).exists() {
                tokio::fs::rename(&src, &dst).await.context(CommonIoSnafu)?;
            }
        }
        Ok(())
    }

    async fn extract() -> Result<()> {
        Self::update_files().await?;
        Self::before_extract().await?;
        Self::perform_extract().await?;
        Self::after_extract().await?;
        Ok(())
    }

    async fn after_extract() -> Result<()> {
        Ok(())
    }
}

#[derive(rust_embed::RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/assets/"]
pub struct Asset;

pub const EXTRACT_SWAGGER_DIR_PATH: &str = "embed_files/swagger";
pub const EXTRACT_DIR_PATH: &str = "embed_files";

impl AssetExtractExt for Asset {}
