#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use rust_embed::RustEmbed;
use tokio::io::AsyncWriteExt;
use tower_http::services::ServeDir;

/// used with swagger openapi
/// eg: I have a swagger.json at path swagger-files/api.json, so I can start a http service for generate swagger
/// ```rust,no_run
/// use awesome_operates::embed::{server_dir, EXTRACT_DIR_PATH};
/// use awesome_operates::swagger::InitSwagger;
/// use axum::{Router, Extension, routing::get, Json, response::{Response, IntoResponse}};
/// use tower::ServiceBuilder;
/// use tower_http::compression::CompressionLayer;
/// use aide::openapi::OpenApi;
/// use aide::transform::TransformOpenApi;
/// use std::sync::Arc;
///
/// async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
///     Json(serde_json::json!(*api)).into_response()
/// }
///
/// fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
///     api.title("数据采集")
/// }
///
/// #[tokio::test]
/// async fn server() -> anyhow::Result<()> {
///     aide::gen::on_error(|error| {
///         println!("{error}")
///     });
///     aide::gen::extract_schemas(true);
///     let mut api = OpenApi::default();
///
///     awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
///     InitSwagger::new(EXTRACT_DIR_PATH, "swagger-init.js", "swagger.html", "../api.json").build().await.unwrap();
///     let app = Router::new()
///         // .api_route("/example", post_with(handlers::example, handlers::example_docs))
///         .nest_service("/docs/", server_dir(EXTRACT_DIR_PATH).await.unwrap())
///         .route("/api.json", get(serve_docs))
///         .finish_api_with(&mut api, api_docs)
///         .layer(ServiceBuilder::new()
///                 .layer(CompressionLayer::new())
///                 .layer(Extension(Arc::new(api))));
///
///     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
/// #    axum::serve(listener, app).await.unwrap();
///     Ok(())
///  }
/// ```
/// finally, you can visit at browser at http://127.0.0.1:3000/docs/ for your swagger
#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/embed_files/"]
pub struct Asset;

pub const EXTRACT_SWAGGER_DIR_PATH: &str = "embed_files/swagger";
pub const EXTRACT_DIR_PATH: &str = "embed_files";

pub async fn server_dir(dir_path: &str) -> anyhow::Result<ServeDir> {
    let dir_path_clone = dir_path.to_owned();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            pre_compress_dir(&dir_path_clone).await;
        });
    });
    Ok(ServeDir::new(dir_path)
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_gzip()
        .precompressed_zstd())
}

/// only used for `pre_compress_dir`
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

/// very time consuming operate, maybe even minitues
/// use `tokio::spawn`
/// ```rust
/// use awesome_operates::embed::pre_compress_dir;
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
        .filter(|e| e.path().is_file() && !e.path().extension().unwrap_or_default().eq("br"))
    {
        multi_compress(entry.path())
            .await
            .unwrap_or_else(|e| tracing::warn!("pre compress failed with `{e:?}`"))
    }
    tracing::info!("pre brotli compress for {dir} over");
}

pub async fn multi_compress(path: &Path) -> anyhow::Result<()> {
    let permissions = tokio::fs::metadata(path).await?;
    #[cfg(unix)]
    if permissions.mode() & 0o200 != 0 {
        tracing::debug!("{} don't has write permission", path.display());
        return Ok(());
    }
    tracing::debug!("pre compress {}", path.display());
    let data = tokio::fs::read(path).await?;
    compress!(BrotliEncoder, "br", data, path);
    compress!(GzipEncoder, "gz", data, path);
    Ok(())
}
