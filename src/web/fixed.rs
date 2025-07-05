use rocket::http::ContentType;
use rust_embed::Embed;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Embed)]
#[folder = "static/web/"]
pub struct Asset;

#[get("/static/<file..>")]
pub fn dist(file: PathBuf) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let filename = file.display().to_string();
    let asset = Asset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some((content_type, asset.data))
}
