#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use tokio::io::AsyncWriteExt;

/// very time consuming operate, maybe even minitues
/// use `tokio::spawn`
/// ```rust
/// use awesome_operates::compress::pre_compress_dir;
///
/// #[tokio::test]
/// async fn compress_all() {
///     tokio::task::spawn_blocking(move || {
///         tokio::runtime::Handle::current().block_on(async move {
///             pre_compress_dir("").await;
///         });
///     });
/// }
/// ```
pub async fn pre_compress_dir(dir: &str) {
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && !["br", "gz"].contains(
                    &e.path()
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                )
        })
    {
        multi_compress(entry.path())
            .await
            .unwrap_or_else(|e| tracing::warn!("pre compress failed with `{e:?}`"))
    }
    tracing::info!("pre brotli compress for {dir} over");
}

/// only used for `pre_compress_dir`
#[macro_export]
macro_rules! compress {
    ($encoder:ident, $extension:expr, $data:expr, $path:expr) => {
        let mut encoder = async_compression::tokio::write::$encoder::with_quality(
            Vec::new(),
            async_compression::Level::Best,
        );
        encoder.write_all(&$data).await?;
        encoder.shutdown().await?;
        let compressed = encoder.into_inner();
        tokio::fs::write(format!("{}.{}", $path.display(), $extension), compressed).await?;
    };
}

pub async fn multi_compress(path: &Path) -> anyhow::Result<()> {
    let permissions = tokio::fs::metadata(path).await?;
    #[cfg(unix)]
    if permissions.mode() & 0o200 == 0 {
        tracing::info!(
            "{} don't has write permission, just skip it, the file permission is `{:#o}`",
            path.display(),
            permissions.mode()
        );
        return Ok(());
    }
    tracing::debug!("pre compress {}", path.display());
    let data = tokio::fs::read(path).await?;
    compress!(BrotliEncoder, "br", data, path);
    compress!(GzipEncoder, "gz", data, path);
    Ok(())
}
