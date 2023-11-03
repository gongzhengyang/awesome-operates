use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[prefix = "embed_files/"]
#[folder = "src/embed_files/"]
pub struct Asset;
