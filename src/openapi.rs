use axum::routing::MethodRouter;
use serde_json::Value;

use crate::method_exchange;

/// load openapi json into route handlers
///
/// ```rust
///  async fn main() {
///     use axum::http::Method;
///     use awesome_operates::router::RequestMatcher;
///
///     let api = tokio::fs::read_to_string("api.json").await.unwrap();
///     let body = serde_json::from_slice(&api.as_bytes())?;
///
///     let route_handles = awesome_operates::openapi::openapi_route_handles(body, "");
///     let mut request_matcher = RequestMatcher::from_route_methods(route_handles);
///     let result = request_matcher.match_request_to_response(Method::GET, "/api/text", None).await;
/// }
/// ```
pub fn openapi_route_handles(openapi: &Value, path_prefix: &str) -> Vec<(String, MethodRouter)> {
    let mut route_handlers = vec![];
    let default_json_value = serde_json::json!(serde_json::Value::Null);
    for (path, operate) in openapi.as_object().unwrap()["paths"]
        .as_object()
        .unwrap()
        .iter()
    {
        let path = path.replace('{', ":").replace('}', "");
        for (method, detail) in operate.as_object().unwrap().iter() {
            let summary = detail
                .get("summary")
                .unwrap_or(&default_json_value)
                .as_str()
                .unwrap_or_default();
            let request_body = &detail["requestBody"]["content"]["application/json"]["schema"]
                ["$ref"]
                .as_str()
                .unwrap_or_default();
            let component = api_component(openapi, request_body);
            let resp = serde_json::json!({
                "path": path,
                "method": method,
                "summary": summary,
                "component": component,
            });
            tracing::debug!(
                "set------[path]: {path}: [method]:{method}: \
            [summary]: {summary}: [request]: {request_body} \
            [component]: {component:?} resp: {resp}"
            );
            route_handlers.push((
                format!("{path_prefix}{path}"),
                method_exchange!(method, &path, resp),
            ));
        }
    }
    route_handlers
}

pub fn api_component<'a>(openapi: &'a Value, component_path: &str) -> Option<&'a Value> {
    if component_path.is_empty() {
        None
    } else {
        openapi.pointer(&component_path.replace('#', ""))
    }
}

#[cfg(test)]
mod tests {
    use hyper::Method;
    use serde_json::Value;

    use crate::openapi::openapi_route_handles;
    use crate::router::{response_to_json, RequestMatcher};

    use super::*;

    #[tokio::test]
    async fn openapi() {
        let openapi = std::fs::read_to_string("src/test_files/openapi.json").unwrap();
        let openapi = serde_json::from_str::<Value>(&openapi).unwrap();
        let route_handles = openapi_route_handles(&openapi);
        let mut request_mather = RequestMatcher::from_route_methods(route_handles);
        for (method, path, expect) in [
            (
                Method::GET,
                "/device/",
                serde_json::json!({
                    "path": "/device/",
                    "method": "get",
                    "summary": "查询设备状态数据 (最多保存历史1000条)",
                    "component": api_component(&openapi, "")
                }),
            ),
            (
                Method::GET,
                "/device/iid/",
                serde_json::json!({
                    "path": "/device/:id/",
                    "method": "get",
                    "summary": "查询设备状态数据 (最多保存历史1000条) id",
                    "component": api_component(&openapi, "")
                }),
            ),
            (
                Method::GET,
                "/device/iid/id2/",
                serde_json::json!({
                    "path": "/device/:id/:id2/",
                    "method": "get",
                    "summary": "查询设备状态数据 (最多保存历史1000条) id id",
                    "component": api_component(&openapi, "")
                }),
            ),
            (
                Method::PUT,
                "/execute/",
                serde_json::json!({
                    "path": "/execute/",
                    "method": "put",
                    "summary": "以root用户执行操作系统命令并获取返回",
                    "component": api_component(&openapi, "#/components/schemas/ExecuteCommand")
                }),
            ),
            (
                Method::PUT,
                "/execute/21-test/",
                serde_json::json!({
                    "path": "/execute/:id/",
                    "method": "put",
                    "summary": "以root用户执行操作系统命令并获取返回 id",
                    "component": api_component(&openapi, "#/components/schemas/ExecuteCommand")
                }),
            ),
        ] {
            let mut resp = response_to_json(
                request_mather
                    .match_request_to_response(method, path, None)
                    .await
                    .unwrap(),
            )
            .await;
            resp.as_object_mut().unwrap().remove("request");
            assert_eq!(expect, resp);
        }
    }
}
