use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};

pub fn page_router() -> Router {
    Router::new().route("/", get(index))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

async fn index() -> impl IntoResponse {
    let index_page = IndexTemplate {};
    HtmlTemplate(index_page)
}

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