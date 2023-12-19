use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Request;
use axum::Json;
use serde_json::Value;

use super::config::{BodyMatch, OpenapiMatchResp};

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
        axum::routing::$method(
            |axum::extract::Path(path_args): axum::extract::Path<
                std::collections::HashMap<String, String>,
            >,
             req: axum::extract::Request<axum::body::Body>| async move {
                $crate::router::handler::handle_openapi_request(path_args, req, $resp).await
            },
        )
    };
}

pub async fn handle_openapi_request(
    path_args: HashMap<String, String>,
    req: Request<Body>,
    mut resp: OpenapiMatchResp,
) -> Json<OpenapiMatchResp> {
    resp.url_args = path_args;
    if let Some(component) = &resp.component {
        let match_body_args = match_body_args(component, req.into_body()).await;
        resp.body_match_list = match_body_args;
    }
    resp.update_formatted_summary();
    Json(resp)
}

/// match body properties field by field with component with body
pub async fn match_body_args(component: &Value, body: Body) -> Vec<BodyMatch> {
    let bytes = http_body_util::BodyExt::collect(body)
        .await
        .unwrap()
        .to_bytes();
    tracing::debug!("handle openapi request receive body len {}", bytes.len());
    let json_body = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        tracing::info!("body transfer is not json for {bytes:?}");
        serde_json::json!({})
    });

    let mut resp = vec![];
    if let Some(properties) = component["properties"].as_object() {
        for (key, value) in properties.iter() {
            let body_value = &json_body[key];
            if !body_value.is_null() {
                resp.push(BodyMatch {
                    key: key.clone(),
                    value: body_value.clone(),
                    description: value["description"].as_str().unwrap_or("").to_owned(),
                    value_type: value["type"].as_str().unwrap_or("").to_owned(),
                });
            }
        }
    }
    resp
}
