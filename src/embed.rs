use async_compression::{tokio::write::BrotliEncoder, Level};
use rust_embed::RustEmbed;
use tokio::io::AsyncWriteExt;
use tower_http::services::ServeDir;

/// used with swagger openapi
/// eg: I have a swagger.json at path swagger-files/api.json, so I can start a http service for generate swagger
/// ```rust,no_run
/// use awesome_operates::embed::server_dir;
/// use awesome_operates::swagger::InitSwagger;
/// use axum::Router;
/// use tower::ServiceBuilder;
/// use tower_http::compression::CompressionLayer;
///
/// #[tokio::test]
/// async fn server() -> anyhow::Result<()> {
///     awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
///     let extract_dir_path = "embed_files/swagger";
///     let app = Router::new()
///         .nest_service("/docs/", server_dir(extract_dir_path).await.unwrap())
///         .nest_service("/swagger-api/", server_dir("swagger-files").await.unwrap())
///         .layer(ServiceBuilder::new().layer(CompressionLayer::new()));
///     InitSwagger::new(extract_dir_path, "swagger-init.js", "swagger.html", "./swagger-api/api.json").build().await.unwrap();
///     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
/// #    axum::serve(listener, app).await.unwrap();
///     Ok(())
///  }
/// ```
/// finally, you can visit at browser at http://127.0.0.1:3000/docs/swagger.html for your swagger
#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/embed_files/"]
pub struct Asset;

pub async fn server_dir(dir_path: &str) -> anyhow::Result<ServeDir> {
    pre_brotli_compress_dir(dir_path).await?;
    Ok(ServeDir::new(dir_path)
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_gzip()
        .precompressed_zstd())
}

pub async fn pre_brotli_compress_dir(dir: &str) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && !e.path().extension().unwrap_or_default().eq("br")
        })
    {
        let path = entry.path();
        tracing::debug!("pre brotli compress {}", path.display());
        let data = tokio::fs::read(path).await?;
        let mut encoder = BrotliEncoder::with_quality(Vec::new(), Level::Best);
        encoder.write_all(&data).await?;
        encoder.shutdown().await?;
        let compressed = encoder.into_inner();
        tokio::fs::write(format!("{}.br", path.display()), compressed).await?;
    }
    Ok(())
}
