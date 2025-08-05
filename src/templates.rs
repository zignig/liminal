// The web page templates.

use askama::Template;
use askama_web::WebTemplate;

use crate::notes::Note;

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

// Notes interface 
#[derive(Template, WebTemplate)]
#[template(path = "notes/notes.html")]
pub struct NotesPageTemplate {
    pub notes: Vec<String>,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/note.html")]
pub struct NotePageTemplate {
    pub title: String,
    pub text: String,
    pub section: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/create.html")]
pub struct NoteCreateTemplate {
    pub section: String,
    pub title_error: bool
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/edit.html")]
pub struct NoteEditTemplate {
    pub title: String,
    pub text: String,
    pub section: String,
}
// End notes interface 

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
