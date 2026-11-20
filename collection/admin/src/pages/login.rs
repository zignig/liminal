//! login pages

use crate::templates::HtmlTemplate;
use askama::Template;
use axum::{extract::Form, response::IntoResponse};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

// Login template
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub section: String,
}

pub async fn login() -> impl IntoResponse {
    let login_page = LoginTemplate {
        section: "".to_string(),
    };
    HtmlTemplate(login_page)
}

pub async fn logout(cookies: Cookies) -> impl IntoResponse {
    cookies.remove(Cookie::new("session_id", ""));
    let login_page = LoginTemplate {
        section: "".to_string(),
    };
    HtmlTemplate(login_page)
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    username: String,
    password: String,
}

pub async fn login_post(cookies: Cookies, Form(payload): Form<Payload>) -> impl IntoResponse {
    let login_page = LoginTemplate {
        section: "".to_string(),
    };
    let _ = payload.password;
    // Do not keep in production
    let cookie = Cookie::build(("session_id", payload.username))
        .path("/")
        .http_only(true)
        .secure(true)
        .build();

    cookies.add(cookie);

    HtmlTemplate(login_page)
}
