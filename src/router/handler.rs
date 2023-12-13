use axum::body::Body;
use axum::extract::Request;
use axum::Json;
use serde_json::Value;

#[macro_export]
macro_rules! method_exchange {
    ($method:expr, $path:expr, $resp:expr) => {
        match $method.to_lowercase().as_str() {
            "get" => $crate::build_method_router!(get, $path, $resp),
            "post" => $crate::build_method_router!(post, $path, $resp),
            "delete" => $crate::build_method_router!(delete, $path, $resp),
            "put" => $crate::build_method_router!(put, $path, $resp),
            "patch" => $crate::build_method_router!(patch, $path, $resp),
            _ => $crate::build_method_router!(get, $path, $resp),
        }
    };
}

#[macro_export]
macro_rules! build_method_router {
    ($method:ident, $path:expr, $resp:expr) => {
        axum::routing::$method(|req: axum::extract::Request<axum::body::Body>| async move {
            $crate::router::handler::handle_openapi_request(req, $resp).await
        })
    };
}

pub async fn handle_openapi_request(req: Request<Body>, mut resp: Value) -> Json<Value> {
    let (parts, body) = req.into_parts();
    let bytes = http_body_util::BodyExt::collect(body)
        .await
        .unwrap()
        .to_bytes();
    tracing::debug!("handle openapi request receive body len {}", bytes.len());
    let json_body = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        tracing::info!("body transfer is not json for {bytes:?}");
        serde_json::json!({})
    });
    resp["request"] = serde_json::json!({
        "method": parts.method.as_str(),
        "path": parts.uri.to_string(),
        "body": json_body
    });
    resp["url_args"] = match_url_openapi_path(
        resp["path_with_prefix"].as_str().unwrap_or_default(),
        &parts.uri.to_string(),
    );
    resp["body_match_list"] = match_body_args(&resp["component"], &json_body);
    Json(resp)
}

/// need to remove prefix in true path request
///  match "/device/:id/:id2/" with "/device/aaa/bbb/?sasajk" one by one
/// into {"id": "aaa", "id2": "bbb"}
pub fn match_url_openapi_path(openapi: &str, path: &str) -> Value {
    let mut resp = serde_json::json!({});
    if let Some(openapi) = openapi.split('?').next() {
        if let Some(path) = path.split('?').next() {
            let openapi_splits = openapi.split('/').collect::<Vec<&str>>();
            let path_splits = path.split('/').collect::<Vec<&str>>();
            for (i, s) in openapi_splits.iter().enumerate() {
                if s.starts_with(':') {
                    resp[s.replace(':', "")] = serde_json::json!(path_splits.get(i));
                }
            }
        }
    }
    serde_json::json!(resp)
}

/// match body properties field by field with component with body
pub fn match_body_args(component: &Value, body: &Value) -> Value {
    tracing::debug!("match body with {body}");
    let mut resp = vec![];
    if let Some(properties) = component["properties"].as_object() {
        for (key, value) in properties.iter() {
            let body_value = &body[key];
            if !body_value.is_null() {
                resp.push(serde_json::json!({
                    "key": key,
                    "value": body_value,
                    "description": value["description"],
                    "type": value["type"]
                }))
            }
        }
    }
    serde_json::json!(resp)
}
