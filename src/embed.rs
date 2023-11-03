use rust_embed::RustEmbed;
use tower_http::services::ServeDir;

/// used with swagger openapi
/// eg: I have a swagger.json at path swagger-files/api.json, so I can start a http service for generate swagger
/// ```rust
/// use awesome_operates::embed::server_dir;
/// use awesome_operates::swagger::InitSwagger;
/// use axum::Router;
///
/// # async fn main() {
///     awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
///     let extract_dir_path = "embed_files/swagger";
///     let app = Router::new()
///         .nest_service("/docs/", server_dir(extract_dir_path))
///         .nest_service("/swagger-api/", server_dir("swagger-files"));
///     InitSwagger::new(extract_dir_path, "swagger-init.js", "swagger.html", "/swagger-api/api.json").build().await.unwrap();
///     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
///     axum::serve(listener, app).await.unwrap()
/// # }
/// ```
/// finally, you can visit at browser at http://127.0.0.1:3000/docs/swagger.html for your swagger
#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/embed_files/"]
pub struct Asset;

pub fn server_dir(dir_path: &str) -> ServeDir {
    ServeDir::new(dir_path)
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_gzip()
        .precompressed_zstd()
}
