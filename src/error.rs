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
}

pub type Result<T> = std::result::Result<T, AppError>;
