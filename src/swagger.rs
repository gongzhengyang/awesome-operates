use snafu::ResultExt;
use std::fmt::Display;

use crate::error::{CommonIoSnafu, Result};

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
///use awesome_operates::server::server_dir;
///use awesome_operates::embed::{EXTRACT_DIR_PATH, EXTRACT_SWAGGER_DIR_PATH, AssetExtractExt};
///use awesome_operates::swagger::InitSwagger;
///use awesome_operates::error::Result;
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
///async fn server() -> Result<()> {
///    aide::gen::on_error(|error| {
///        println!("{error}")
///    });
///    aide::gen::extract_schemas(true);
///    let mut api = OpenApi::default();
///
///    awesome_operates::embed::Asset::extract().await?;
///    InitSwagger::new(EXTRACT_SWAGGER_DIR_PATH, "swagger-init.js", "swagger.html", "../api.json").build().await.unwrap();
///    let app = ApiRouter::new()
///        .nest_service("/docs/", server_dir(EXTRACT_DIR_PATH).await)
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

pub struct InitSwagger {
    file_prefix: String,
    pub js_filename: String,
    pub index_html_filename: String,
    pub json_uri: String,
}

/// generate swagger *.html and *initializer.js file with prefix
/// the `json_url` just as a value in *initializer.js file
/// eg.
/// this will generate embed_files/swagger/index.html and
/// embed_files/swagger/swagger-initializer.js(/swagger-json/api.json will be the key url value)
/// ```rust,no_run
/// use awesome_operates::swagger::InitSwagger;
/// use awesome_operates::error::Result;
///
/// #[tokio::test]
/// async fn openapi_write() -> Result<()> {
///     awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
///     InitSwagger::new(
///         "embed_files/swagger/",
///         "swagger-initializer.js",
///         "index.html",
///         "/swagger-json/api.json"
///     ).build().await.unwrap();
///     Ok(())
/// }
/// ```
impl InitSwagger {
    pub fn new<T>(prefix: T, js_filename: T, index_html_filename: T, json_url: T) -> Self
    where
        T: Display,
    {
        InitSwagger {
            file_prefix: prefix.to_string(),
            js_filename: js_filename.to_string(),
            index_html_filename: index_html_filename.to_string(),
            json_uri: json_url.to_string(),
        }
    }

    pub fn js_filepath(&self) -> String {
        let prefix = std::path::Path::new(&self.file_prefix);
        prefix.join(&self.js_filename).to_str().unwrap().to_owned()
    }

    pub fn index_html_filepath(&self) -> String {
        let prefix = std::path::Path::new(&self.file_prefix);
        prefix
            .join(&self.index_html_filename)
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub async fn build(&self) -> Result<()> {
        self.rewrite_swagger_index_html().await?;
        self.rewrite_swagger_initializer_js().await?;
        Ok(())
    }

    pub async fn rewrite_swagger_index_html(&self) -> Result<()> {
        let index_html = format!(
            r#"<!-- HTML for static distribution bundle build -->
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="./swagger-ui.css" />
    <link rel="stylesheet" type="text/css" href="index.css" />
    <link rel="icon" type="image/png" href="./favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="./favicon-16x16.png" sizes="16x16" />
  </head>

  <body>
    <div id="swagger-ui"></div>
    <script src="./swagger-ui-bundle.js" charset="UTF-8"> </script>
    <script src="./swagger-ui-standalone-preset.js" charset="UTF-8"> </script>
    <script src="./{}" charset="UTF-8"> </script>
  </body>
</html>
"#,
            self.js_filename
        );
        tracing::info!(
            "write swagger index at path: {}",
            self.index_html_filepath()
        );
        tokio::fs::write(&self.index_html_filepath(), index_html)
            .await
            .context(CommonIoSnafu)?;
        Ok(())
    }

    pub async fn rewrite_swagger_initializer_js(&self) -> Result<()> {
        let js = format!(
            r#"window.onload = function() {{
  //<editor-fold desc="Changeable Configuration Block">

  // the following lines will be replaced by docker/configurator, when it runs in a docker-container
  window.ui = SwaggerUIBundle({{
    url: "{}",
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
      SwaggerUIBundle.plugins.DownloadUrl
    ],
    layout: "StandaloneLayout",
    defaultModelRendering: "model",
    defaultModelsExpandDepth: 1,
    defaultModelExpandDepth: 10,
    validatorUrl: "localhost",
  }});

  //</editor-fold>
}};"#,
            &self.json_uri
        );
        tracing::info!("write js initializer path: {}", self.js_filepath());
        tokio::fs::write(&self.js_filepath(), js)
            .await
            .context(CommonIoSnafu)?;
        Ok(())
    }
}
