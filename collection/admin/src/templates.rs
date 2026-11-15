use askama::Template;
use axum::{
    Router,
    extract::Form,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

pub fn page_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login).post(login_post))
        .route("/logout", get(logout))
        .route("/index", get(index))
        .route("/workload", get(workload))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    username: Option<String>,
}

async fn index(jar: CookieJar) -> impl IntoResponse {
    let mut authed = None;
    if let Some(sess) = jar
        .get("session_id")
        .map(|cookie| cookie.value().to_owned())
    {
        authed = Some(sess.to_string());
    };
    let index_page = IndexTemplate { username: authed };
    HtmlTemplate(index_page)
}

#[derive(Template)]
#[template(path = "workload.html")]
pub struct WorkloadTemplate;

async fn workload() -> impl IntoResponse {
    let workload_page = WorkloadTemplate {};
    HtmlTemplate(workload_page)
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate;

async fn login() -> impl IntoResponse {
    let login_page = LoginTemplate {};
    HtmlTemplate(login_page)
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar_copy = jar.remove(Cookie::from("session_id"));
    let login_page = LoginTemplate {};
    (jar_copy, HtmlTemplate(login_page))
}

#[derive(Deserialize, Debug)]
struct Payload {
    username: String,
    password: String,
}

async fn login_post(jar: CookieJar, Form(payload): Form<Payload>) -> impl IntoResponse {
    let login_page = LoginTemplate {};
    // Do not keep in production
    let cookie = Cookie::build(("session_id", payload.username))
        .path("/")
        .http_only(true)
        .secure(true)
        .build();

    let updated_jar = jar.add(cookie);
    (updated_jar, HtmlTemplate(login_page))
}
// Template responder
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
