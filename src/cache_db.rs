// use std::path::PathBuf;
// use async_trait::async_trait;
// use serde::de::DeserializeOwned;
// use serde::Serialize;
// use snafu::ResultExt;
// use moka::future::Cache;
// use crate::error::{
//     Result, CommonIoSnafu
// };
//
// #[async_trait]
// pub trait CacheDbExt: Default + Serialize + DeserializeOwned {
//     async fn load_default() {
//         let cache =
//     }
//
//     async fn save(self) -> Result<()> {
//         tokio::fs::write(Self::filepath(),
//                          serde_json::json!(&self).to_string()).await.context(CommonIoSnafu)?;
//         *Self::load_default().await.lock().await = self;
//         Ok(())
//     }
//
//     fn filepath() -> PathBuf {
//         // PathBuf::from_iter([constants::PROXY_AGENT_FS_DIR, "manage"])
//     }
//
// }
