use snafu::ResultExt;
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt};

use crate::error::{CommonIoSnafu, LogFileBuildSnafu, Result, TracingSetGlobalSnafu};

/// Usage
/// ```rust
/// use awesome_operates::log::tracing_both_file_stdout;
/// use std::time::Duration;
/// #[tokio::main]
///  async fn main() {
///     // this is very important, must receive worker_guard value
///     // or file writer will take no reflect
///     let _guard = tracing_both_file_stdout("target", "", "agent.log").await;
///     // let _guard = tracing_both_file_stdout("logs", "", "agent.log").await;
///     tokio::time::sleep(Duration::from_secs(1)).await;
///     tokio::spawn(async {
///         tracing::info!("中文 abcd 1234 ?wajkasd[]{{}}【】---------");
///     });
///  }
/// ```
pub async fn tracing_both_file_stdout(
    log_dir: impl AsRef<Path>,
    log_file_prefix: impl Into<String>,
    log_file_suffix: impl Into<String>,
) -> Result<WorkerGuard> {
    tokio::fs::create_dir_all(&log_dir)
        .await
        .context(CommonIoSnafu)?;
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(log_file_prefix)
        .filename_suffix(log_file_suffix)
        .build(log_dir)
        .context(LogFileBuildSnafu)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let collector = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_ansi(false)
                .with_writer(std::io::stdout),
        )
        .with(fmt::Layer::new().with_ansi(false).with_writer(non_blocking));
    tracing::subscriber::set_global_default(collector).context(TracingSetGlobalSnafu)?;
    Ok(guard)
}
