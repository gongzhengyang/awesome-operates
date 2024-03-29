use tower_http::services::ServeDir;

use crate::compress::pre_compress_dir;

pub async fn server_dir(dir_path: &str) -> ServeDir {
    let dir_path_clone = dir_path.to_owned();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            pre_compress_dir(&dir_path_clone).await;
        });
    });
    ServeDir::new(dir_path)
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_gzip()
        .precompressed_zstd()
}
