use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Request;
use axum::Json;

use super::config::OpenapiMatchResp;

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
    resp.match_body_args(req.into_body()).await;
    resp.update_formatted_summary();
    Json(resp)
}
