use axum::{
    body::Body,
    http::{Method, Request},
    routing::MethodRouter,
    Router,
};
use once_cell::sync::Lazy;
use serde_json::Value;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use tokio::sync::RwLock;
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

pub static GLOBAL_PREFIX_OPENAPI: Lazy<RwLock<HashMap<String, Value>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

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
        let path_prefix_cloned = path_prefix.to_owned();
        let openapi_cloned = openapi.clone();
        tokio::spawn(async move {
            GLOBAL_PREFIX_OPENAPI
                .write()
                .await
                .insert(path_prefix_cloned, openapi_cloned);
        });
        let mut route_handlers = vec![];

        for (path, operate) in iter_object(openapi, "paths")? {
            let path = path.replace('{', ":").replace('}', "");
            for (method, detail) in operate
                .as_object()
                .context(OptionNoneSnafu)?
                .iter()
                .filter(|(_, obj)| obj.is_object())
            {
                let (module, openapi_log) = Self::fetch_openapi_module_log(detail);
                let resp = OpenapiMatchResp {
                    openapi_path: path.clone(),
                    method: method.clone(),
                    openapi_log,
                    module,
                    component: Self::api_component(
                        openapi,
                        detail.pointer("/requestBody/content/application~1json/schema/$ref"),
                    )
                    .cloned(),
                    prefix: path_prefix.to_owned(),
                    ..Default::default()
                };
                let path_with_prefix = format!("{}{path}", path_prefix.trim_end_matches('/'));
                tracing::debug!(
                    "generate path_with_prefix {path_with_prefix} {:?}",
                    resp.component
                );
                route_handlers.push((path_with_prefix, method_exchange!(method, &path, resp)));
            }
        }
        Ok(route_handlers)
    }

    /// fetch the line in summary or first line in description starts with `[`
    pub fn fetch_openapi_module_log(detail: &Value) -> (String, String) {
        for key in ["summary", "description"] {
            if let Some((k, v)) = Self::fetch_openapi_log_by_key(detail, key) {
                return (k, v);
            }
        }
        ("".to_owned(), "".to_owned())
    }

    fn fetch_openapi_log_by_key(detail: &Value, key: &str) -> Option<(String, String)> {
        let value = detail.get(key)?.as_str()?.split('\n').next()?;
        if !value.trim().starts_with('[') {
            return None;
        }
        let (module, log) = value.split_once(']')?;
        Some((
            module.replace('[', "").trim().to_owned(),
            log.trim().to_owned(),
        ))
    }

    pub fn api_component<'a>(
        openapi: &'a Value,
        component_path: Option<&Value>,
    ) -> Option<&'a Value> {
        openapi.pointer(component_path?.as_str()?.trim_start_matches('#'))
    }

    pub async fn match_request_to_response(
        &mut self,
        method: Method,
        path: &str,
        body: Option<Body>,
    ) -> Result<OpenapiMatchResp> {
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

        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .context(AxumSnafu)?
            .to_bytes();
        let resp = serde_json::from_slice::<OpenapiMatchResp>(&bytes).context(SerdeJsonSnafu)?;
        tracing::debug!("match resp {resp:?}");
        Ok(resp)
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
