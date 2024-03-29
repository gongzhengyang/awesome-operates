use std::sync::Arc;

use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use aide::transform::TransformOpenApi;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use awesome_operates::embed::{AssetExtractExt, EXTRACT_SWAGGER_DIR_PATH};
use awesome_operates::error::Result;
use awesome_operates::server::server_dir;
use awesome_operates::swagger::InitSwagger;

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
    Json(serde_json::json!(*api)).into_response()
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("example openapi/swagger")
}

#[derive(Deserialize, JsonSchema, Serialize)]
struct User {
    pub name: String,
}

#[derive(rust_embed::RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/bin/"]
pub struct Asset;

async fn example() -> Json<User> {
    Json(User {
        name: "hello".to_owned(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    server().await.unwrap();
}

async fn server() -> Result<()> {
    aide::gen::on_error(|error| println!("{error}"));
    aide::gen::extract_schemas(true);
    let mut api = OpenApi::default();
    awesome_operates::embed::Asset::extract().await?;
    InitSwagger::new(
        EXTRACT_SWAGGER_DIR_PATH,
        "swagger-init.js",
        "index.html",
        "../api.json",
    )
    .build()
    .await
    .unwrap();
    let app = ApiRouter::new()
        .api_route("/hello", aide::axum::routing::get(example))
        .nest_service("/swagger/", server_dir(EXTRACT_SWAGGER_DIR_PATH).await)
        .route("/api.json", get(serve_docs))
        .finish_api_with(&mut api, api_docs)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(Extension(Arc::new(api))),
        );
    let addr = "0.0.0.0:3000";
    tracing::info!("visit http://{addr}/swagger/ for swagger");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
