use snafu::ResultExt;
use std::path::Path;

use crate::error::{CommonIoSnafu, Result};

pub async fn create_file_parent_dir(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context(CommonIoSnafu)?;
    }
    Ok(())
}
