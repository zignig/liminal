//!  The index page

use crate::templates::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;
use tower_cookies::Cookies;

// The template object

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub username: Option<String>,
    pub section: String,
}

pub async fn index(cookies: Cookies) -> impl IntoResponse {
    let mut authed = None;
    if let Some(sess) = cookies
        .get("session_id")
        .map(|cookie| cookie.value().to_owned())
    {
        authed = Some(sess.to_string());
    };
    let index_page = IndexTemplate {
        username: authed,
        section: "index".to_string(),
    };
    HtmlTemplate(index_page)
}
