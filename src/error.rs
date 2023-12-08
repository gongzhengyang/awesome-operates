use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use snafu::prelude::*;
use snafu::Location;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppError {
    #[snafu(display("Option value is None"))]
    OptionNone { location: Location },

    #[snafu(display("axum error: {}", source))]
    Axum {
        source: axum::Error,
        location: Location,
    },

    #[snafu(display("proxy request error `{}`", source))]
    RequestProxy {
        location: Location,
        source: reqwest::Error,
    },

    #[snafu(display("request build error `{}`", source))]
    HttpRequestBuild {
        location: Location,
        source: http::Error,
    },

    #[snafu(display("request build from parts error `{}`", source))]
    UriFromParts {
        location: Location,
        source: http::uri::InvalidUriParts,
    },

    #[snafu(display("request body error `{}`", source))]
    RequestBodyRead {
        location: Location,
        source: reqwest::Error,
    },

    #[snafu(display("binary cannot be execute for filepath {}", filepath))]
    BinaryCannotBeExecute {
        location: Location,
        filepath: String,
    },

    #[snafu(display("common io error {}", source))]
    CommonIo {
        location: Location,
        source: std::io::Error,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = StatusCode::INTERNAL_SERVER_ERROR;
        tracing::error!("error happened: {self:?}\n display error: {self}");
        (
            status_code,
            Json(serde_json::json!({
                "message": format!("{}", self)
            })),
        )
            .into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
