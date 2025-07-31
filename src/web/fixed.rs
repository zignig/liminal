//! Serves static files, with a cache header.
//! 

use rocket::http::uri::fmt::FromUriParam;
use rocket::http::{ContentType, Header};
use rocket::response::Responder;
use rocket::{Response, get};
use rust_embed::{Embed, EmbeddedFile};

use std::borrow::Cow;
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
    let file  = PathBuf::from_str("img/favicon.ico").unwrap();
    let asset = Asset::get(&file.display().to_string())?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some(CacheControl::new(content_type, asset))
}