// This is a wrapper set around iroh-docs
// Based upon the tauri to example
// With alterations...
// https://github.com/n0-computer/iroh-examples/blob/main/tauri-todos/src-tauri/src/todos.rs

use std::{cmp::Reverse, str::FromStr, sync::Arc};

use anyhow::{Context, Result, anyhow, bail, ensure};
use bytes::Bytes;
use chrono::Utc;
use iroh_blobs::BlobsProtocol;
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
            text: String::from(""),
            created: 0,
            is_delete: false,
            id,
        }
    }
}

// Notes outer
#[derive(Debug, Clone)]
pub struct Notes(Arc<Inner>);

// Inner hiding behind the arc
#[derive(Debug, Clone)]
pub struct Inner {
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

        Ok(Self(Arc::new(Inner {
            blobs,
            docs,
            doc,
            ticket,
            author,
        })))
    }

    pub async fn from_id(id: [u8; 32], blobs: BlobsProtocol, docs: Docs) -> Result<Self> {
        let doc = docs.open(id.into()).await?;
        let doc = match doc {
            Some(doc) => doc,
            None => return Err(anyhow!("Doc does not exist")),
        };
        let ticket = doc.share(ShareMode::Write, Default::default()).await?;
        // TODO , save the author key in config ( just create a new one for now)
        let author = docs.author_create().await?;
        Ok(Self(Arc::new(Inner {
            blobs,
            docs,
            doc,
            ticket,
            author,
        })))
    }

    pub fn id(&self) -> [u8; 32] {
        self.0.doc.id().to_bytes()
    }

    pub fn ticket(&self) -> String {
        self.0.ticket.to_string()
    }

    pub async fn doc_subscribe(&self) -> Result<impl Stream<Item = Result<LiveEvent>> + use<>> {
        self.0.doc.subscribe().await
    }

    pub async fn create(&self, id: String, text: String) -> Result<()> {
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
        let entries = self.0.doc.get_many(Query::single_latest_per_key()).await?;
        let mut notes = Vec::new();
        // TODO remove once entries are unpin !
        tokio::pin!(entries);
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let note = self.note_from_entry(&entry).await?;
            if !note.is_delete {
                notes.push(note)
            }
            // notes.push(note);
        }
        notes.sort_by_key(|n| Reverse(n.created));
        Ok(notes)
    }

    pub async fn get_note(&self, id: String) -> Result<Note> {
        let entry_option = self
            .0
            .doc
            .get_one(Query::single_latest_per_key().key_exact(&id))
            .await?;
        match entry_option {
            Some(entry) => self.note_from_entry(&entry).await,
            None => Ok(Note::missing_note(id.clone())),
        }
    }

    pub async fn update_note(&self, id: String, text: String) -> Result<()> {
        if text.len() > MAX_TEXT_LEN {
            bail!("text is too long, max size is {MAX_TEXT_LEN}");
        };
        let mut note = self.get_note(id.clone()).await?;
        note.text = text;
        self.update_bytes(id, note).await
    }

    pub async fn fix_title(&self, id: String) -> Result<()> {
        {
            let mut note = self.get_note(id.clone()).await?;
            note.id = "__".to_string();
            self.update_bytes(id, note).await
        }
    }

    pub async fn delete_hidden(&self) -> Result<()> {
        let entries = self.0.doc.get_many(Query::single_latest_per_key()).await?;
        // delete hidden docs ; ( admin move )
        tokio::pin!(entries);
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let note = self.note_from_entry(&entry).await?;
            if !note.is_delete || note.id == "__".to_string() {
                let _ = self.0.doc.del(self.0.author, note.id).await;
            }
        }
        Ok(())
    }

    pub async fn set_delete(&self, id: String) -> Result<()> {
        let mut note = self.get_note(id.clone()).await?;
        note.is_delete = !note.is_delete;
        self.update_bytes(id, note).await
    }

    // Doc data manipulation
    async fn insert_bytes(&self, key: impl AsRef<[u8]>, value: Bytes) -> Result<()> {
        self.0
            .doc
            .set_bytes(self.0.author, key.as_ref().to_vec(), value)
            .await?;
        Ok(())
    }

    async fn update_bytes(&self, key: impl AsRef<[u8]>, note: Note) -> Result<()> {
        let content = note.as_bytes()?;
        self.insert_bytes(key, content).await
    }

    async fn note_from_entry(&self, entry: &Entry) -> Result<Note> {
        let id = String::from_utf8(entry.key().to_owned()).context("invalid key")?;
        match self.0.blobs.get_bytes(entry.content_hash()).await {
            Ok(b) => Note::from_bytes(b),
            Err(_) => Ok(Note::missing_note(id)),
        }
    }

    // End direct doc manipulation
}
