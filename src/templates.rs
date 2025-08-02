// The web page templates.

use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct HomePageTemplate {
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "files.html")]
pub struct  FilePageTemplate {
    pub items: Vec<String>,
    pub path: String,
    pub segments: Vec<String>,
    pub prefixes: Vec<String>,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes.html")]
pub struct NotesPageTemplate {
    pub notes: Vec<String>,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "note.html")]
pub struct NotePageTemplate {
    pub keys: Vec<String>,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "network.html")]
pub struct NetworkPageTemplate {
    pub nodes: Vec<String>,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "node.html")]
pub struct NodePageTemplate {
    pub node_id: String,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "gltfview.html")]
pub struct GltfPageTemplate {
    pub path: String,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "login.html")]
pub struct LoginPageTemplate {
    pub section: String
}
