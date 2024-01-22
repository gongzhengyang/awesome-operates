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

    #[snafu(display("str parse error {}", source))]
    MethodStrParseError {
        source: http::method::InvalidMethod,
        location: Location,
    },

    #[snafu(display("serde_json {}", source))]
    SerdeJson {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("zip extract {}", source))]
    ZipExtract {
        source: zip::result::ZipError,
        location: Location,
    },

    #[snafu(display("log file build InitError {}", source))]
    LogFileBuild {
        source: tracing_appender::rolling::InitError,
        location: Location,
    },

    #[snafu(display("tracing set global failed {}", source))]
    TracingSetGlobal {
        source: tracing::subscriber::SetGlobalDefaultError,
        location: Location,
    },

    #[snafu(display("serde urlencoded error {source}"))]
    SerdeUrlEncodedSer {
        source: serde_urlencoded::ser::Error,
        location: Location,
    },

    #[snafu(display("serde urlencoded error de {source}"))]
    SerdeUrlEncodedDe {
        source: serde_urlencoded::de::Error,
        location: Location,
    },

    #[snafu(display("invalid url {source}"))]
    InvalidUrl {
        source: http::uri::InvalidUri,
        location: Location,
    },
    #[snafu(display("invalid uri parts {source}"))]
    InvalidUriParts {
        source: http::uri::InvalidUriParts,
        location: Location,
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
