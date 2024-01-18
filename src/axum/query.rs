// use async_trait::async_trait;
// use axum::extract::{FromRequestParts, Query, rejection::*};
// use serde::de::DeserializeOwned;
// use http::{request::Parts, Uri};
//
// #[derive(Debug, Clone, Copy, Default)]
// pub struct QueryTrimEmpty<T>(pub T);
//
// #[async_trait]
// impl<T, S> FromRequestParts<S> for QueryTrimEmpty<T>
//     where
//         T: DeserializeOwned,
//         S: Send + Sync,
// {
//     type Rejection = QueryRejection;
//
//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         let query = &parts.uri.query().unwrap_or_default();
//         match serde_urlencoded::from_str(query) {
//             Ok(q) => Ok(Self(q)),
//             Err(e) => {
//                 Err(Self::Rejection)
//             }
//         }
//     }
// }
