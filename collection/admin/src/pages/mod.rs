/// The actual routed pages , broken out into a module.
///
use axum::{Router, extract::Form, response::IntoResponse, routing::get};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use crate::templates::{HtmlTemplate, IndexTemplate, LoginTemplate, WorkloadTemplate};

pub fn page_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login).post(login_post))
        .route("/logout", get(logout))
        .route("/index", get(index))
        .route("/workload", get(workload))
}

async fn index(cookies: Cookies) -> impl IntoResponse {
    let mut authed = None;
    if let Some(sess) = cookies
        .get("session_id")
        .map(|cookie| cookie.value().to_owned())
    {
        authed = Some(sess.to_string());
    };
    let index_page = IndexTemplate {
        username: authed,
        section: "index".to_string()
    };
    HtmlTemplate(index_page)
}

async fn workload() -> impl IntoResponse {
    let workload_page = WorkloadTemplate {
        section: "workload".to_string(),
    };
    HtmlTemplate(workload_page)
}

async fn login() -> impl IntoResponse {
    let login_page = LoginTemplate { section: "".to_string() };
    HtmlTemplate(login_page)
}

async fn logout(cookies: Cookies) -> impl IntoResponse {
    cookies.remove(Cookie::new("session_id", ""));
    let login_page = LoginTemplate {
        section: "".to_string(),
    };
    HtmlTemplate(login_page)
}

#[derive(Deserialize, Debug)]
struct Payload {
    username: String,
    password: String,
}

async fn login_post(cookies: Cookies, Form(payload): Form<Payload>) -> impl IntoResponse {
    let login_page = LoginTemplate {
        section: "".to_string(),
    };
    // Do not keep in production
    let cookie = Cookie::build(("session_id", payload.username))
        .path("/")
        .http_only(true)
        .secure(true)
        .build();

    cookies.add(cookie);

    HtmlTemplate(login_page)
}
