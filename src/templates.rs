

use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct HomePageTemplate {
}

#[derive(Template, WebTemplate)]
#[template(path = "files.html")]
pub struct FilePageTemplate{
    pub items: Vec<String>,
    pub path: String,
    pub segments: Vec<String>,
    pub prefixes: Vec<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes.html")]
pub struct NotesPageTemplate {
}


#[derive(Template, WebTemplate)]
#[template(path = "network.html")]
pub struct NetworkPageTemplate {
    pub nodes: Vec<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "gltfview.html")]
pub struct GltfPageTemplate {
    path: String
}