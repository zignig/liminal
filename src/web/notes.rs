use std::ptr::dangling;
use std::str::FromStr;

use crate::templates::{NotePageTemplate, NotesPageTemplate};
use anyhow::{Result, anyhow};

use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{CapabilityKind, Entry, NamespaceId};

use n0_future::StreamExt;
use rocket::State;
use rocket::mtls::oid::asn1_rs::nom::AsBytes;
use rocket::response::Responder;

async fn get_all_docs(docs: &Docs) -> Result<Vec<(NamespaceId, CapabilityKind)>> {
    // Create a new doc here for good fun
    let all_doc: Vec<_> = docs.list().await?.try_collect().await?;
    Ok(all_doc)
}

#[get("/notes")]
pub async fn show_notes<'r>(docs: &State<Docs>) -> impl Responder<'r, 'static> {
    let res = get_all_docs(&docs).await;
    let docs = match res {
        Ok(docs) => docs
            .iter()
            .map(|(n, _)| data_encoding::BASE32_NOPAD.encode(n.as_bytes()))
            .collect(),
        Err(_) => vec![],
    };

    NotesPageTemplate {
        notes: docs,
        section: "notes".to_string(),
    }
}

// TODO doc id's should not be used here
async fn get_doc(doc_id: &str, docs: &Docs) -> Result<Vec<Entry>> {
    let default_author = docs.author_default().await?;
    let dec_doc_id: [u8; 32] = data_encoding::BASE32_NOPAD
        .decode(doc_id.as_bytes())?
        .try_into()
        .expect("message fail");
    let id = NamespaceId::from(&dec_doc_id);
    let doc_op = docs.open(id).await?;
    if let Some(doc) = doc_op {
        doc.set_bytes(default_author, "fnord", "this is a test").await?;
        let query = Query::all().build();
        let entries: Vec<Entry> = doc.get_many(query).await?.try_collect().await?;
        return Ok(entries);
    }
    Err(anyhow!("doc fail"))
}

#[get("/notes/<doc_id>")]
pub async fn show_note<'r>(doc_id: &str, docs: &State<Docs>) -> impl Responder<'r, 'static> {
    let doc_res = get_doc(doc_id, docs).await;
    let keys = match doc_res {
        Ok(keys) => keys
            .iter()
            .map(|k| k.content_hash().to_hex())
            .collect(),
        Err(e) => {
            println!("{:#?}", e);
            vec![]
        }
    };
    println!("{:#?}", keys);
    NotePageTemplate {
        keys: keys,
        section: "notes".to_string(),
    }
}
