use async_trait::async_trait;
use rust_embed::RustEmbed;

use crate::error::Result;

#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/assets/"]
pub struct Asset;

pub const EXTRACT_SWAGGER_DIR_PATH: &str = "embed_files/swagger";
pub const EXTRACT_DIR_PATH: &str = "embed_files";

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
            let filepath = file.as_ref().clone();
            let file = <$asset>::get(filepath).unwrap().data.clone();
            $crate::helper::write_filepath_with_data(filepath, file).await?;
        }
    };
}

#[async_trait]
pub trait AssetExtractExt: RustEmbed {
    async fn before_extract() -> Result<()> {
        Ok(())
    }

    async fn perform_extract() -> Result<()> {
        extract_all_files!(Self);
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
