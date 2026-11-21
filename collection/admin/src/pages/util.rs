//!  The util page

use crate::templates::HtmlTemplate;
use axum::response::IntoResponse;

use crate::bindgen::wasi::keyvalue::store::open;
use super::index::IndexTemplate;

// The template object

pub async fn util() -> impl IntoResponse {
    // create some keys in the KV space
    let data = open("testing").expect("no keyspace");
    for (i,j) in (0..100).enumerate(){
        let _ = data.set(&i.to_string(), j.to_string().as_bytes());
    }

    let index_page = IndexTemplate {
        username: None,
        section: "index".to_string(),
    };
    HtmlTemplate(index_page)
}