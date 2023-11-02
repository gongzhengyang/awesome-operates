use axum::body::{Body, HttpBody};
use axum::http::Request;
use axum::routing::MethodRouter;
use axum::Router;
use tower::Service;
use tower::ServiceExt;

#[derive(Default)]
pub struct RequestMatcher {
    // pub route_method: Vec<String, MethodRouter>,
    router: Router,
}

impl RequestMatcher {
    fn build_router(&mut self, route_methods: Vec<String, MethodRouter>) {
        for (path, resp) in route_methods {
            self.router = self.router.route(path, resp);
        }
    }

    async fn match_request(&mut self, request: Request<Body>) {
        let request = Request::builder()
            .uri("/test/sa/tec")
            .body(Body::empty())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        let (parts, mut body) = response.into_parts();
        let body = body.data().await;
    }
}

#[cfg(test)]
mod tests {}
