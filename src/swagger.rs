use std::fmt::Display;

pub struct InitSwagger {
    file_prefix: String,
    pub js_filename: String,
    pub index_html_filename: String,
    pub json_uri: String,
}

/// ```rust,no_run
/// use awesome_operates::swagger::InitSwagger;
///
/// #[tokio::test]
/// async fn openapi_write() -> anyhow::Result<()> {
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

    pub async fn build(&self) -> anyhow::Result<()> {
        self.rewrite_swagger_index_html().await?;
        self.rewrite_swagger_initializer_js().await?;
        Ok(())
    }

    pub async fn rewrite_swagger_index_html(&self) -> anyhow::Result<()> {
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
        tokio::fs::write(&self.index_html_filepath(), index_html).await?;
        Ok(())
    }

    pub async fn rewrite_swagger_initializer_js(&self) -> anyhow::Result<()> {
        let js = format!(
            r#"window.onload = function() {{
  //<editor-fold desc="Changeable Configuration Block">

  // the following lines will be replaced by docker/configurator, when it runs in a docker-container
  window.ui = SwaggerUIBundle({{
    url: "{}",
    dom_id: '#swagger-ui',
    deepLinking: false,
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
  }});

  //</editor-fold>
}};"#,
            &self.json_uri
        );
        tracing::info!("write js initializer path: {}", self.js_filepath());
        tokio::fs::write(&self.js_filepath(), js).await?;
        Ok(())
    }
}
