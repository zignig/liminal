// Notes web interface

use crate::notes::Notes;
use crate::templates::{NoteCreateTemplate, NoteEditTemplate, NotePageTemplate, NotesPageTemplate};

use rocket::State;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::response::{Redirect, Responder};



pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Notes Browser", |rocket| async {
        rocket.mount(
            "/",
            routes![
                show_notes,
                show_note,
                create_note,
                make_note,
                edit_note,
                update_note,
                delete_note
            ],
        )
    })
}

#[get("/notes")]
pub async fn show_notes<'r>(notes: &State<Notes>) -> impl Responder<'r, 'static> {
    let note_list_res = notes.get_notes().await;
    let items = match note_list_res {
        Ok(notes) => notes.iter().map(|n| n.id.clone()).collect(),
        Err(_) => todo!(),
    };
    NotesPageTemplate {
        notes: items,
        section: "notes".to_string(),
    }
}

#[get("/notes/show/<doc_id>")]
pub async fn show_note<'r>(doc_id: &str, notes: &State<Notes>) -> impl Responder<'r, 'static> {
    let doc_res = notes.get_note(doc_id.to_string()).await;
    let (title, value) = match doc_res {
        Ok(doc) => (doc.id, doc.text),
        Err(_) => ("Bad Note".to_string(), "Note error".to_string()),
    };

    NotePageTemplate {
        title: title,
        text: value,
        section: "notes".to_string(),
    }
}

#[get("/notes/edit/<doc_id>")]
pub async fn edit_note<'r>(doc_id: &str, notes: &State<Notes>) -> impl Responder<'r, 'static> {
    let doc_res = notes.get_note(doc_id.to_string()).await;
    let (title, value) = match doc_res {
        Ok(doc) => (doc.id, doc.text),
        Err(_) => ("Bad Note".to_string(), "Note error".to_string()),
    };

    NoteEditTemplate {
        title: title,
        text: value,
        section: "notes".to_string(),
    }
}

#[get("/notes/create")]
pub async fn create_note<'r>() -> impl Responder<'r, 'static> {
    NoteCreateTemplate {
        section: "notes".to_string(),
    }
}

// The node form data
#[derive(FromForm)]
pub struct NoteCreate<'v> {
    title: &'v str,
    text: &'v str,
}

#[post("/notes/create", data = "<note_data>")]
pub async fn make_note<'r>(
    note_data: Form<NoteCreate<'_>>,
    notes: &State<Notes>,
) -> impl Responder<'r, 'static> {
    let clean_title: String =  note_data.title.chars().filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace()).collect();
    let res = notes
        .add(clean_title.clone(), note_data.text.to_string())
        .await;
    match res {
        Ok(_) => Redirect::to(uri!(show_note(clean_title))),
        Err(e) => {
            println!("{:#?}", e);
            Redirect::to(uri!(create_note()))
        }
    }
}

#[post("/notes/update", data = "<note_data>")]
pub async fn update_note<'r>(
    note_data: Form<NoteCreate<'_>>,
    notes: &State<Notes>,
) -> impl Responder<'r, 'static> {
    let res = notes
        .update_note(note_data.title.to_string(), note_data.text.to_string())
        .await;
    match res {
        Ok(_) => Redirect::to(uri!(show_note(note_data.title))),
        Err(_) => Redirect::to(uri!(create_note())),
    }
}

#[get("/notes/delete/<doc_id>")]
pub async fn delete_note<'r>(doc_id: &str, notes: &State<Notes>) -> impl Responder<'r, 'static> {
    let res = notes.set_delete(doc_id.to_string()).await;
    match res {
        Ok(_) => Redirect::to(uri!(show_notes)),
        Err(_) => Redirect::to(uri!(show_notes)),
    }
}
