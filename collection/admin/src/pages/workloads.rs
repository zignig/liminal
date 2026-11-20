//!  The workloads page

use crate::templates::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;

// The template object
#[derive(Template)]
#[template(path = "workload.html")]
pub struct WorkloadTemplate{ 
    pub section: String
}

pub async fn workload() -> impl IntoResponse {
    let workload_page = WorkloadTemplate {
        section: "workload".to_string(),
    };
    HtmlTemplate(workload_page)
}