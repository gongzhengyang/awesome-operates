use async_trait::async_trait;
use axum::response::IntoResponse;
use rust_embed::RustEmbed;
use snafu::{ErrorCompat, IntoError, NoneError, ResultExt};
use crate::error::{AppError, CommonIoSnafu};

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
            let filepath = file.as_ref();
            let file = <$asset>::get(filepath).unwrap().data;
            $crate::helper::write_filepath_with_data(filepath, file).await?;
        }
    };
}

// #[async_trait]
// pub trait AssetExtractExt: RustEmbed {
//     type Error: snafu::Error;
//
//     async fn before_extract() -> Result<(), Self::Error> {
//         Ok(())
//     }
//
//     async fn perform_extract() -> Result<(), Self::Error> {
//         // extract_all_files!(Self);
//         tokio::fs::read("").await.context(CommonIoSnafu)?;
//         Ok(())
//     }
//
//     async fn extract() -> Result<(), Self::Error> {
//         Self::before_extract().await?;
//         Self::perform_extract().await?;
//         Self::after_extract().await?;
//         Ok(())
//     }
//
//     async fn after_extract() -> Result<(), Self::Error> {
//         Ok(())
//     }
// }
