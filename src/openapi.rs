use axum::routing::MethodRouter;
use serde_json::Value;

use crate::method_exchange;

pub fn openapi_json_parse(openapi: Value) {
    let mut route_handlers: Vec<(String, MethodRouter)> = vec![];
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
                .unwrap_or_else(|| &default_json_value)
                .as_str()
                .unwrap_or_default();
            let request_body = &detail["requestBody"]["content"]["application/json"]["schema"]
                ["$ref"]
                .as_str()
                .unwrap_or_default();
            let component = if request_body.is_empty() {
                &default_json_value
            } else {
                openapi
                    .pointer(&request_body.replace('#', ""))
                    .unwrap_or_else(|| &default_json_value)
            };
            let resp = serde_json::json!({
                "path": path,
                "method": method,
                "summary": summary,
                "component": component,
            });
            route_handlers.push((path.clone(), method_exchange!(method, &path, resp)));
            tracing::debug!("{path}: {method}: {summary}: {request_body} {component}");
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::openapi::openapi_json_parse;

    #[test]
    fn openapi() {
        let value = std::fs::read_to_string("src/test_files/openapi.json").unwrap();
        let value = serde_json::from_str::<Value>(&value).unwrap();
        openapi_json_parse(value)
    }
}
