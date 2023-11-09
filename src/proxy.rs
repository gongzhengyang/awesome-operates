use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::Request;
use axum::http;
use axum::response::{IntoResponse, Response};
use http::uri::{Parts, Scheme};
use http::Uri;
use snafu::ResultExt;
use tower::make::Shared;

use crate::error::{RequestBodyReadSnafu, RequestProxySnafu, Result, UriFromPartsSnafu};

#[async_trait]
pub trait HttpProxy: 'static {
    fn listen_addr() -> &'static str;

    fn target_addr() -> String;

    fn proxy_timeout() -> u64 {
        10
    }

    fn default_schema() -> Scheme {
        Scheme::HTTP
    }

    async fn service_proxy(req: Request<hyper::Body>) -> Result<Response> {
        let uri = req.uri();

        let mut parts = Parts::default();
        parts.scheme = Some(Self::default_schema());
        parts.authority = Some(Self::target_addr().parse().unwrap());
        parts.path_and_query = uri.path_and_query().cloned();
        tracing::debug!("receive proxy: {parts:?} headers: {:?}", req.headers());
        let changed_uri = Uri::from_parts(parts).context(UriFromPartsSnafu)?;

        let client = reqwest::Client::new();
        let resp = client
            .request(req.method().clone(), changed_uri.to_string())
            .headers(req.headers().clone())
            .body(req.into_body())
            .timeout(Duration::from_secs(Self::proxy_timeout()))
            .send()
            .await
            .context(RequestProxySnafu)?;

        let status_code = resp.status().clone();
        let headers = resp.headers().clone();
        tracing::debug!("proxy result {changed_uri:?} with resp: {status_code} {headers:?}");
        let body = resp.bytes().await.context(RequestBodyReadSnafu)?;
        let mut res = Body::from(body).into_response();
        *res.status_mut() = status_code;
        *res.headers_mut() = headers;
        Ok(res)
    }

    async fn proxy_server() {
        tracing::info!("listening on {}", Self::listen_addr());
        let service = tower::service_fn(Self::service_proxy);
        hyper::Server::bind(&Self::listen_addr().parse().unwrap())
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .serve(Shared::new(service))
            .await
            .unwrap();
    }
}
