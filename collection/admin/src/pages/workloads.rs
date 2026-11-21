//!  The workloads page

use crate::templates::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;

use crate::bindgen::wasi::keyvalue::store::open;

// The template object
#[derive(Template)]
#[template(path = "workload.html")]
pub struct WorkloadTemplate{ 
    pub section: String,
    pub items: Vec<String>,
}

pub async fn workload() -> impl IntoResponse {
    
    // get a list of the keys just for testing.

    let data = open("testing").expect("no keyspace");
    let items = data.list_keys(None).expect("no keys");
    let data = items.keys.clone();
    let workload_page = WorkloadTemplate {
        section: "workload".to_string(),
        items: data,
    };
    HtmlTemplate(workload_page)
}