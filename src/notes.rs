// This is a wrapper set around iroh-docs
//  Based upon the tauri to example
// https://github.com/n0-computer/iroh-examples/blob/main/tauri-todos/src-tauri/src/todos.rs

use std::str::FromStr;

use anyhow::{Context, Result, bail, ensure};
use bytes::Bytes;
use chrono::Utc;
use iroh_blobs::{api::blobs::Blobs, BlobsProtocol};
use iroh_docs::{
    AuthorId, DocTicket, Entry,
    api::{Doc, protocol::ShareMode},
    engine::LiveEvent,
    protocol::Docs,
    store::Query,
};

use n0_future::{Stream, StreamExt};

use serde::{Deserialize, Serialize};

// Individual notes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub created: i64,
    pub is_delete: bool,
}

const MAX_NOTE_SIZE: usize = 2 * 1024;
const MAX_TEXT_LEN: usize = 2 * 1000;

impl Note {
    fn from_bytes(bytes: Bytes) -> anyhow::Result<Self> {
        let note = serde_json::from_slice(&bytes).context("invalid json")?;
        Ok(note)
    }

    fn as_bytes(&self) -> anyhow::Result<Bytes> {
        let buf = serde_json::to_vec(self)?;
        ensure!(buf.len() < MAX_NOTE_SIZE, "todo too large");
        Ok(buf.into())
    }

    fn missing_note(id: String) -> Self {
        Self {
            text: String::from("Missing Content"),
            created: 0,
            is_delete: false,
            id,
        }
    }
}

pub struct Notes {
    blobs: BlobsProtocol,
    docs: Docs,
    doc: Doc,
    ticket: DocTicket,
    author: AuthorId,
}

impl Notes {
    pub async fn new(ticket: Option<String>, blobs: BlobsProtocol, docs: Docs) -> Result<Self> {
        let author = docs.author_create().await?;
        let doc = match ticket {
            Some(ticket) => {
                let ticket = DocTicket::from_str(&ticket)?;
                docs.import(ticket).await?
            }
            None => docs.create().await?,
        };
        let ticket = doc.share(ShareMode::Write, Default::default()).await?;

        Ok(Self {
            blobs,
            docs,
            doc,
            ticket,
            author,
        })
    }

    pub fn ticket(&self) -> String {
        self.ticket.to_string()
    }

    pub async fn doc_subscribe(&self) -> Result<impl Stream<Item = Result<LiveEvent>> + use<>> {
        self.doc.subscribe().await
    }

    pub async fn add(&mut self, id: String, text: String) -> Result<()> {
        if text.len() > MAX_TEXT_LEN {
            bail!("text is too long, max size is {MAX_TEXT_LEN}");
        };
        let created = Utc::now().timestamp();
        let note = Note {
            id: id.clone(),
            text,
            created,
            is_delete: false,
        };
        self.insert_bytes(id.as_bytes(), note.as_bytes()?).await
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>> {
        let entries = self.doc.get_many(Query::single_latest_per_key()).await?;
        let mut notes = Vec::new();
        // TODO remove once entries are unpin ! 
        tokio::pin!(entries);
        while let Some(entry) = entries.next().await {
                let entry = entry?;
                let note = self.note_from_entry(&entry).await?;
                if !note.is_delete {
                    notes.push(note)
                }
        }
        notes.sort_by_key(|n| n.created);
        Ok(notes)
    }

    async fn insert_bytes(&self, key: impl AsRef<[u8]>, value: Bytes) -> Result<()> {
        self.doc
            .set_bytes(self.author, key.as_ref().to_vec(), value)
            .await?;
        Ok(())
    }

    async fn note_from_entry(&self, entry: &Entry) -> Result<Note> {
        let id = String::from_utf8(entry.key().to_owned()).context("invalid key")?;
        match self.blobs.get_bytes(entry.content_hash()).await {
            Ok(b) => Note::from_bytes(b),
            Err(_) => Ok(Note::missing_note(id)),
        }
    }
}
