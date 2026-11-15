/// Template objects and base handler
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub username: Option<String>,
    pub section: String,
}

#[derive(Template)]
#[template(path = "workload.html")]
pub struct WorkloadTemplate{ 
    pub section: String
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate{
    pub section: String
}

// Template responder
pub struct HtmlTemplate<T>(pub T);

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
