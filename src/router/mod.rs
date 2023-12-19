use axum::{
    body::Body,
    http::{self, Method, Request},
    response::Response,
    routing::MethodRouter,
    Router,
};
use serde_json::Value;
use snafu::{OptionExt, ResultExt};
use tower::{Service, ServiceExt};

pub use config::{BodyMatch, OpenapiMatchResp};

use crate::error::{AxumSnafu, MethodStrParseSnafu, OptionNoneSnafu, Result, SerdeJsonSnafu};
use crate::helper::iter_object;
use crate::method_exchange;

mod config;
mod handler;
#[cfg(test)]
mod tests;

/// if used as static, use `tokio::sync::Mutex`, this is very importment
/// ```rust,no_run
/// use std::sync::Arc;
///
/// use axum::http::Method;
/// use once_cell::sync::Lazy;
/// use tokio::sync::Mutex;
/// use serde_json::Value;
///
/// use awesome_operates::router::RequestMatcher;
///
/// static REQUEST_MATCHER: Lazy<Arc<Mutex<RequestMatcher>>> = Lazy::new(||
/// { Arc::new(Mutex::new(RequestMatcher::default())) });
///
/// #[tokio::test]
/// async fn matcher() {
///     let api = tokio::fs::read_to_string("api.json").await.unwrap();
///     let body = serde_json::from_str::<Value>(&api).unwrap();
///
///     let mut request_matcher = RequestMatcher::from_openapi(&body, "").unwrap();
///     // use directly
///     request_matcher.match_request_to_response(Method::GET, "/api/test", None).await.unwrap();
///
///     // or use global
///     *REQUEST_MATCHER.lock().await = request_matcher;
///     REQUEST_MATCHER.lock().await.match_request_to_response(Method::GET, "/api/test", None).await.unwrap();
/// }
/// ```
#[derive(Default)]
pub struct RequestMatcher {
    pub router: Router,
}

impl RequestMatcher {
    pub fn from_openapi(openapi: &Value, path_prefix: &str) -> Result<Self> {
        let route_handles = Self::openapi_route_handles(openapi, path_prefix)?;
        Ok(RequestMatcher::from_route_methods(route_handles))
    }

    pub fn from_route_methods(route_methods: Vec<(String, MethodRouter)>) -> Self {
        let mut router = Router::new();
        for (path, resp) in route_methods {
            router = router.route(path.as_ref(), resp);
        }
        RequestMatcher { router }
    }

    /// path_prefix is like "/sys-layer", "/api/v1"
    /// openapi refer to `src/test_files/openapi.json`
    pub fn openapi_route_handles(
        openapi: &Value,
        path_prefix: &str,
    ) -> Result<Vec<(String, MethodRouter)>> {
        let mut route_handlers = vec![];
        let default_summary = serde_json::json!("");
        for (path, operate) in iter_object(openapi, "paths")? {
            let path = path.replace('{', ":").replace('}', "");
            for (method, detail) in operate.as_object().context(OptionNoneSnafu)?.iter() {
                if !detail.is_object() {
                    continue;
                }
                let summary = detail.get("summary");
                let component = Self::api_component(
                    openapi,
                    detail.pointer("/requestBody/content/application~1json/schema/$ref"),
                );
                let path_with_prefix = format!("{}{path}", path_prefix.trim_end_matches('/'));
                let resp = OpenapiMatchResp {
                    openapi_path: path.clone(),
                    method: method.clone(),
                    openapi_summary: summary
                        .unwrap_or(&default_summary)
                        .as_str()
                        .unwrap_or("")
                        .to_owned(),
                    component: component.cloned(),
                    path_with_prefix: path_with_prefix.clone(),
                    ..Default::default()
                };
                route_handlers.push((path_with_prefix, method_exchange!(method, &path, resp)));
            }
        }
        Ok(route_handlers)
    }

    pub fn api_component<'a>(
        openapi: &'a Value,
        component_path: Option<&Value>,
    ) -> Option<&'a Value> {
        if let Some(path) = component_path {
            if let Some(p) = path.as_str() {
                if p.starts_with('#') {
                    return openapi.pointer(&p.replace('#', ""));
                }
            }
        }
        None
    }

    pub async fn match_request_to_response(
        &mut self,
        method: Method,
        path: &str,
        body: Option<Body>,
    ) -> Result<Response> {
        // this line is very important
        let method = method
            .as_str()
            .to_uppercase()
            .parse()
            .context(MethodStrParseSnafu)?;
        tracing::debug!(
            "match request [method]{} [path]:{} body:[{body:?}] ",
            method,
            path
        );
        let request = Self::build_request(method, path, body);
        tracing::debug!("match request before {request:?}");
        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        tracing::debug!("match api with result status: {}", response.status());
        Ok(response)
    }

    pub async fn match_request_to_json_response(
        &mut self,
        method: Method,
        path: &str,
        body: Option<Body>,
    ) -> Result<Value> {
        let response = self.match_request_to_response(method, path, body).await?;
        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .context(AxumSnafu)?
            .to_bytes();
        Ok(serde_json::from_slice(&bytes).context(SerdeJsonSnafu)?)
    }

    pub fn build_request(method: Method, path: &str, body: Option<Body>) -> Request<Body> {
        let body = body.unwrap_or_default();
        Request::builder()
            .method(method)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(path)
            .body(body)
            .unwrap()
    }
}
