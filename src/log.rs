use std::path::Path;

use snafu::ResultExt;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
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
///     let _guard = tracing_both_file_stdout("target", "", "agent.log", None).await;
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
    rotation: Option<Rotation>,
) -> Result<WorkerGuard> {
    let (non_blocking, guard) =
        tracing_with_file(log_dir, log_file_prefix, log_file_suffix, rotation).await?;
    #[cfg(windows)]
    let ansi_enabled = false;
    #[cfg(unix)]
    let ansi_enabled = true;

    let collector = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_ansi(ansi_enabled)
                .with_writer(std::io::stdout),
        )
        .with(fmt::Layer::new().with_ansi(false).with_writer(non_blocking));
    tracing::subscriber::set_global_default(collector).context(TracingSetGlobalSnafu)?;
    Ok(guard)
}

pub async fn tracing_with_file(
    log_dir: impl AsRef<Path>,
    log_file_prefix: impl Into<String>,
    log_file_suffix: impl Into<String>,
    rotation: Option<Rotation>,
) -> Result<(NonBlocking, WorkerGuard)> {
    tokio::fs::create_dir_all(&log_dir)
        .await
        .context(CommonIoSnafu)?;
    let file_appender = RollingFileAppender::builder()
        .rotation(rotation.unwrap_or(Rotation::DAILY))
        .filename_prefix(log_file_prefix)
        .filename_suffix(log_file_suffix)
        .build(log_dir)
        .context(LogFileBuildSnafu)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    Ok((non_blocking, guard))
}

// pub trait CollectorFmtLayer<S>
//     where
//         S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>
// {
//     type L: Layer<S>;
//     fn fmt_layer(&self) -> Self::L;
// }
//
// pub struct StdoutLayer;
//
// impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> CollectorFmtLayer<S> for StdoutLayer {
//     type L = fmt::Layer<S, DefaultFields, Format, fn() -> Stdout>;
//
//     fn fmt_layer(&self) -> Self::L {
//         #[cfg(windows)]
//             let ansi_enabled = false;
//         #[cfg(unix)]
//             let ansi_enabled = true;
//         fmt::Layer::new().with_ansi(ansi_enabled).with_writer(std::io::stdout)
//     }
// }
//
// pub struct FileFmtLayer;
//
// impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> CollectorFmtLayer<S> for FileFmtLayer {
//     type L = fmt::Layer<S>;
//
//     fn fmt_layer(&self) -> Self::L {
//         fmt::Layer::new().with_ansi(false)
//     }
// }
