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
pub struct FilePageTemplate {
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
    pub note: Note,
    pub text: String, 
    pub section: String,
    pub notes: Vec<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/create.html")]
pub struct NoteCreateTemplate {
    pub section: String,
    pub title_error: bool,
    pub notes: Vec<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "notes/edit.html")]
pub struct NoteEditTemplate {
    pub note: Note,
    pub section: String,
    pub notes: Vec<String>,
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
    pub section: String,
    pub node_id: String
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
    pub section: String,
}

// icon listing
#[derive(Template, WebTemplate)]
#[template(path = "icons.html")]
pub struct IconsPageTemplate {
    pub section: String,
    pub icons: Vec<String>,
}
