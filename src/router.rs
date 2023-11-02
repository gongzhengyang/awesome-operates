use axum::{
    body::Body,
    http::{self, Request},
    response::Response,
    routing::MethodRouter,
    Router,
};
use hyper::Method;
use serde_json::Value;
use tower::{Service, ServiceExt};

#[derive(Default)]
pub struct RequestMatcher {
    pub router: Router,
}

impl RequestMatcher {
    pub fn from_route_methods(route_methods: Vec<(&str, MethodRouter)>) -> Self {
        let mut router = Router::new();
        for (path, resp) in route_methods {
            router = router.route(path, resp);
        }
        RequestMatcher { router }
    }

    pub async fn match_request_to_response(
        &mut self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> anyhow::Result<Response> {
        let request = Self::build_request(method, path, body);
        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await?
            .call(request)
            .await?;
        Ok(response)
    }

    pub fn build_request(method: Method, path: &str, body: Option<Value>) -> Request<Body> {
        let body = if let Some(body) = body {
            Body::from(serde_json::to_vec(&body).unwrap())
        } else {
            Body::empty()
        };
        Request::builder()
            .method(method)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(path)
            .body(body)
            .unwrap()
    }
}

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

// ///  get "/api/:id/test/:id2" ["a", "b"] -> get(|Path(paths): Path<Vec<String>>| async move {}}
#[macro_export]
macro_rules! build_method_router {
    ($method:ident, $path:expr, $resp:expr) => {
        axum::routing::$method(
            |axum::extract::Path(paths): axum::extract::Path<Vec<String>>| async move {
                let mut resp = $resp;
                resp["path_args"] = serde_json::json!(paths);
                axum::Json(resp)
            },
        )
    };
}

#[cfg(test)]
mod tests {
    use axum::extract::{Json, Path};
    use axum::{http::Method, routing::get};

    use super::*;

    async fn response_to_str(response: Response) -> String {
        String::from_utf8(
            hyper::body::to_bytes(response.into_body())
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap()
    }

    fn app() -> RequestMatcher {
        let route_handlers =
            [
                (
                    "/api",
                    get(|| async { "GET api" }).post(|| async { "POST api" }),
                ),
                (
                    "/api/:id/test",
                    get(|Path(test): Path<String>| async move { format!("GET api {test}") })
                        .patch(|Path(id): Path<String>| async move { format!("PATCH api {id}") })
                        .put(
                            |Path(id): Path<String>, Json(body): Json<Value>| async move {
                                format!("PUT with {id} body: {body}")
                            },
                        ),
                ),
                (
                    "/api/:id/:ccc/test",
                    get(|Path(paths): Path<Vec<String>>| async move {
                        format!("double path {paths:?}")
                    }),
                ),
                (
                    "/api/:id/test-single",
                    get(|Path(id): Path<String>| async move { format!("single path {id}") }),
                ),
                (
                    "/api/ccc",
                    get(|Path(paths): Path<Vec<String>>| async move {
                        format!("GET api with / {paths:?}")
                    }),
                ),
                (
                    "/macro/:id/sa/:id2",
                    method_exchange!("get", "/macro", serde_json::json!({"default": ""})),
                ),
            ]
            .to_vec();
        RequestMatcher::from_route_methods(route_handlers)
    }

    #[tokio::test]
    async fn basic() {
        let mut app = app();
        for (method, path, expect, body) in [
            (Method::GET, "/api", "GET api", None),
            (Method::POST, "/api", "POST api", None),
            (Method::GET, "/api/fake-id/test", "GET api fake-id", None),
            (
                Method::PATCH,
                "/api/fake-id/test",
                "PATCH api fake-id",
                None,
            ),
            (Method::GET, "/api/33/test", "GET api 33", None),
            (
                Method::PUT,
                "/api/body-id/test",
                "PUT with body-id body: {\"name\":\"test\"}",
                Some(serde_json::json!({"name": "test"})),
            ),
            (Method::GET, "/api/ccc", "GET api with / []", None),
            (Method::GET, "/not-found", "", None),
            (
                Method::GET,
                "/api/cc/dd/test",
                "double path [\"cc\", \"dd\"]",
                None,
            ),
            (
                Method::GET,
                "/api/cccc/test-single",
                "single path cccc",
                None,
            ),
            (
                Method::GET,
                "/macro/1/sa/222",
                "{\"default\":\"\",\"path_args\":[\"1\",\"222\"]}",
                None,
            ),
        ] {
            let response = response_to_str(
                app.match_request_to_response(method, path, body)
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(response, expect);
        }
    }
}
