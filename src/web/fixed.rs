//! Serves static files, with a cache header.
//!

use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::{Response, get};
use rust_embed::{Embed, EmbeddedFile};

use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Embed)]
#[folder = "static/web/"]
pub struct Asset;

pub struct CacheControl {
    content_type: ContentType,
    asset: EmbeddedFile,
}

impl CacheControl {
    fn new(content_type: ContentType, asset: EmbeddedFile) -> Self {
        Self {
            content_type: content_type,
            asset: asset,
        }
    }
}

impl<'r> Responder<'r, 'static> for CacheControl {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let res = self.asset.data.respond_to(request)?;
        let val = Response::build()
            .join(res)
            .header(self.content_type)
            .raw_header("Cache-Control", "max-age=86400")
            .ok();
        val
    }
}

#[get("/static/<file..>")]
pub fn dist(file: PathBuf) -> Option<CacheControl> {
    let filename = file.display().to_string();
    let asset = Asset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some(CacheControl::new(content_type, asset))
}

#[get("/favicon.ico")]
pub fn favicon() -> Option<CacheControl> {
    let file = PathBuf::from_str("img/favicon.ico").unwrap();
    let asset = Asset::get(&file.display().to_string())?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some(CacheControl::new(content_type, asset))
}

// fa5 icons 
// stolen from 
// https://gist.githubusercontent.com/sakalauskas/b0c5049d5dc349713a82f1cb2a30b2fa/raw/ce34182e1ac873b0185b03731ec8bd47072c8e0e/FontAwesome-v5.0.9-Free.json
