use std::str::FromStr;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::request::Parts;
use http::uri::PathAndQuery;
use http::Uri;
use snafu::{OptionExt, ResultExt};

use crate::error::{
    InvalidUriPartsSnafu, InvalidUrlSnafu, OptionNoneSnafu, Result, SerdeUrlEncodedDeSnafu,
    SerdeUrlEncodedSerSnafu,
};

pub async fn query_trim_empty_items_middleware(request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    tracing::debug!("original url {:?}", parts.uri.path_and_query());
    query_trim(&mut parts).unwrap_or_default();
    tracing::debug!("trim query {:?}", parts.uri.path_and_query());
    let request = Request::from_parts(parts, body);
    next.run(request).await
}

#[inline]
fn query_trim(parts: &mut Parts) -> Result<()> {
    let query = parts.uri.query().context(OptionNoneSnafu)?;
    let values = serde_urlencoded::from_str::<Vec<(String, String)>>(query)
        .context(SerdeUrlEncodedDeSnafu)?;
    let mut true_filters = vec![];
    for (k, v) in &values {
        if !v.is_empty() {
            true_filters.push((k, v));
        }
    }
    let trim_query = serde_urlencoded::to_string(true_filters).context(SerdeUrlEncodedSerSnafu)?;
    let mut uri_parts = parts.uri.clone().into_parts();
    let path = parts.uri.path();
    uri_parts.path_and_query =
        Some(PathAndQuery::from_str(&format!("{path}?{trim_query}")).context(InvalidUrlSnafu)?);
    parts.uri = Uri::from_parts(uri_parts).context(InvalidUriPartsSnafu)?;
    Ok(())
}
