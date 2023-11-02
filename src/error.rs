use snafu::prelude::*;
use snafu::Location;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppError {
    #[snafu(display("Option value is None"))]
    OptionNone { location: Location },
}
