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
                let mut resp = $resp;
                let (parts, body) = req.into_parts();
                let bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
                let json_body = serde_json::from_slice(&bytes).unwrap_or_else(|_| serde_json::json!({}));
                resp["request"] = serde_json::json!({
                    "method": parts.method.as_str(),
                    "path": parts.uri.to_string(),
                    // "headers": parts.headers,
                    "body": json_body
                });
                axum::Json(resp)
            },
        )
    };
}
