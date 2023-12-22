use rust_embed::RustEmbed;
use tower_http::services::ServeDir;

use crate::compress::pre_compress_dir;

/// used with swagger openapi
/// eg: I have a swagger.json at path swagger-files/api.json, so I can start a http service for generate swagger
/// ```rust,no_run
/// use std::sync::Arc;
///
///use aide::axum::ApiRouter;
///use aide::openapi::OpenApi;
///use aide::transform::TransformOpenApi;
///use axum::{Extension, Json, response::{IntoResponse, Response}, routing::get};
///use tower::ServiceBuilder;
///use tower_http::compression::CompressionLayer;
///
///use awesome_operates::embed::{EXTRACT_DIR_PATH, EXTRACT_SWAGGER_DIR_PATH, server_dir};
///use awesome_operates::swagger::InitSwagger;
///
///async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
///    Json(serde_json::json!(*api)).into_response()
///}
///
///fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
///    api.title("数据采集")
///}
///
///#[tokio::main]
///async fn main() {
///    tracing_subscriber::fmt::init();
///  //  server().await.unwrap();
///}
///
///async fn server() -> anyhow::Result<()> {
///    aide::gen::on_error(|error| {
///        println!("{error}")
///    });
///    aide::gen::extract_schemas(true);
///    let mut api = OpenApi::default();
///
///    awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
///    InitSwagger::new(EXTRACT_SWAGGER_DIR_PATH, "swagger-init.js", "swagger.html", "../api.json").build().await.unwrap();
///    let app = ApiRouter::new()
///        .nest_service("/docs/", server_dir(EXTRACT_DIR_PATH).await.unwrap())
///        .route("/api.json", get(serve_docs))
///        .finish_api_with(&mut api, api_docs)
///        .layer(ServiceBuilder::new()
///            .layer(CompressionLayer::new())
///            .layer(Extension(Arc::new(api))));
///
///    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
///    axum::serve(listener, app).await.unwrap();
///    Ok(())
///}
/// ```
/// finally, you can visit at browser at http://127.0.0.1:3000/docs/ for your swagger
#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/assets/"]
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
            $crate::write_filepath_with_data(filepath, file).await?;
        }
    };
}
