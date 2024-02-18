use async_trait::async_trait;
use rust_embed::RustEmbed;

use crate::error::Result;
use crate::helper;

#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/assets/"]
pub struct Asset;
impl AssetExtractExt for Asset {}

pub const EXTRACT_SWAGGER_DIR_PATH: &str = "embed_files/swagger";
pub const EXTRACT_DIR_PATH: &str = "embed_files";

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
/// #   Asset::extract().await?;
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
            if helper::is_current_running_newer(filepath).is_ok_and(|v| v) {
                let file = Self::get(filepath).unwrap().data.clone();
                helper::write_filepath_with_data(filepath, file)?;
            } else {
                tracing::debug!("skip {filepath} because it newer");
            }
        }
        Ok(())
    }

    async fn extract() -> Result<()> {
        Self::before_extract().await?;
        Self::perform_extract().await?;
        Self::after_extract().await?;
        Ok(())
    }

    async fn after_extract() -> Result<()> {
        Ok(())
    }
}
