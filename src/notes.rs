// This is a wrapper set around iroh-docs
// Based upon the tauri to example
// With alterations...
// https://github.com/n0-computer/iroh-examples/blob/main/tauri-todos/src-tauri/src/todos.rs

// It turns out that iroh-docs uses a _weird_ prefix key system (for shared docs apparently)
// This means that if you write a key that is a prefix of any other it will destroy keys
// I don't think this is sane... but woteva.

// This means that all keys in the docs system need to have a delimiter at the end
// so they don't prefix
// when writing keys it appending a null byte on the end will fix this
// as  per https://github.com/n0-computer/iroh-docs/issues/55
// So ... when keys are written or read they need to have a null byte added
// or removed as they come in and out of docs. Insane...

use std::{cmp::Reverse, str::FromStr, sync::Arc};

use anyhow::{Context, Result, anyhow, bail, ensure};
use bytes::Bytes;
use chrono::{Local, Utc};
use iroh_blobs::{BlobsProtocol, format::collection::Collection};
use iroh_docs::{
    AuthorId, DocTicket, Entry,
    api::{
        Doc,
        protocol::{AddrInfoOptions, ShareMode},
    },
    engine::LiveEvent,
    protocol::Docs,
    store::Query,
};

use n0_future::{FuturesUnordered, Stream, StreamExt, task};
use serde::{Deserialize, Serialize};

// Individual notes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub created: i64,
    pub updated: i64,
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
            updated: 0,
            is_delete: false,
            id,
        }
    }

    pub fn bad_note() -> Self {
        Self {
            text: String::from("bad_note"),
            created: 0,
            updated: 0,
            is_delete: false,
            id: String::from("bad_note"),
        }
    }
}

// All the keys need to be null byte extended , ? I know right

// Notes outer
#[derive(Debug, Clone)]
pub struct Notes(Arc<Inner>);

// Inner hiding behind the arc
#[derive(Debug, Clone)]
pub struct Inner {
    blobs: BlobsProtocol,
    _docs: Docs,
    doc: Doc,
    ticket: DocTicket,
    author: AuthorId,
}

impl Notes {
    // Create a new docset
    pub async fn new(
        ticket: Option<String>,
        author: AuthorId,
        blobs: BlobsProtocol,
        docs: Docs,
    ) -> Result<Self> {
        let author = author;
        let doc = match ticket {
            Some(ticket) => {
                let ticket = DocTicket::from_str(&ticket)?;
                docs.import(ticket).await?
            }
            None => docs.create().await?,
        };
        let ticket = doc
            .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
            .await?;

        Ok(Self(Arc::new(Inner {
            blobs,
            _docs: docs.clone(),
            doc,
            ticket,
            author,
        })))
    }

    pub async fn from_id(
        id: [u8; 32],
        author: AuthorId,
        blobs: BlobsProtocol,
        docs: Docs,
    ) -> Result<Self> {
        let doc = docs.open(id.into()).await?;
        let doc = match doc {
            Some(doc) => doc,
            None => return Err(anyhow!("Doc does not exist")),
        };
        let ticket = doc
            .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
            .await?;
        let author = author;
        Ok(Self(Arc::new(Inner {
            blobs,
            _docs: docs.clone(),
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

    pub async fn leave(&self) -> Result<()> {
        self.0.doc.leave().await?;
        Ok(())
    }

    pub async fn share(&self) -> Result<()> {
        self.0.doc.start_sync(vec![]).await?;
        Ok(())
    }

    pub async fn doc_subscribe(&self) -> Result<impl Stream<Item = Result<LiveEvent>> + use<>> {
        self.0.doc.subscribe().await
    }

    pub async fn run(&self) -> Result<()> {
        let events = self.doc_subscribe().await?;
        task::spawn(info_loop(events));
        Ok(())
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
            updated: created,
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
        // println!("{:#?}",notes);
        notes.sort_by_key(|n| Reverse(n.updated));
        Ok(notes)
    }

    // Just get a vec
    pub async fn get_note_vec(&self) -> Vec<String> {
        let note_list_res = self.get_notes().await;
        let items = match note_list_res {
            Ok(notes) => notes.iter().map(|n| n.id.clone()).collect(),
            Err(e) => vec![format!("{e}")],
        };
        items
    }

    pub async fn get_note(&self, id: String) -> Result<Note> {
        let mut ex_key = id.as_bytes().to_vec();
        ex_key.push(0);
        let entry_option = self
            .0
            .doc
            .get_one(Query::single_latest_per_key().key_exact(&ex_key))
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
        let note_res = self.get_note(id.clone()).await;
        let mut note = match note_res {
            Ok(note) => note,
            Err(_) => Note::missing_note("missing".to_string()),
        };
        note.text = text;
        note.updated = Utc::now().timestamp();
        let res = self.update_bytes(id.as_bytes(), note).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_hidden(&self) -> Result<()> {
        let entries = self.0.doc.get_many(Query::single_latest_per_key()).await?;
        tokio::pin!(entries);
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let note = self.note_from_entry(&entry).await?;
            if note.is_delete {
                println!("deleting => {:#?}", note);
                let val = self
                    .0
                    .doc
                    .del(self.0.author, entry.key().to_owned())
                    .await?;
                println!("{:?} nodes deleted", val);
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

    // TODO null byte the id for weird reasons
    async fn insert_bytes(&self, key: impl AsRef<[u8]>, value: Bytes) -> Result<()> {
        // null byte exend the key
        let mut ex_key = key.as_ref().to_vec();
        // add the null byte, why ??
        ex_key.push(0);
        // harrahs b5 to not do this
        self.0.doc.set_bytes(self.0.author, ex_key, value).await?;
        Ok(())
    }

    async fn update_bytes(&self, key: impl AsRef<[u8]>, note: Note) -> Result<()> {
        let content = note.as_bytes()?;
        self.insert_bytes(key, content).await
    }

    async fn note_from_entry(&self, entry: &Entry) -> Result<Note> {
        let mut id = String::from_utf8(entry.key().to_owned()).context("invalid key")?;
        match self.0.blobs.get_bytes(entry.content_hash()).await {
            Ok(b) => Note::from_bytes(b),
            Err(e) => {
                id.pop();
                println!("{:#?} this is probably it {:#?}", &id, e);
                return Ok(Note::missing_note(id));
            }
        }
    }

    // Save out the docs as date stamped .md files
    pub async fn bounce_down(&self) -> Result<()> {
        let entries = self.0.doc.get_many(Query::single_latest_per_key()).await?;
        let mut notes = Vec::new();
        // TODO remove once entries are unpin !
        tokio::pin!(entries);
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let note = self.note_from_entry(&entry).await?;
            if !note.is_delete {
                let h = self.0.blobs.add_bytes(note.text).await?.hash;
                let mut file_name = Local::now().format("notes/%Y/%m/%d/").to_string();
                file_name.push_str(&note.id.clone());
                // add markdown file extension for good measure
                file_name.push_str(".md");
                notes.push((format!("{file_name}"), h));
            }
        }
        // print!("{:#?}", notes);
        let col = notes.into_iter().collect::<Collection>();
        let col_hash = col.store(&self.0.blobs).await?;
        // tag it for replication
        let dt = Local::now().timestamp();
        self.0
            .blobs
            .tags()
            .set(format!("notes-{}", dt), &col_hash)
            .await?;
        // println!("notes bounce down {:?}", col_hash);
        Ok(())
    }

    pub async fn bounce_up(&self, id: &str) -> Result<()> {
        let tag = match self.0.blobs.tags().get(id).await? {
            Some(tag) => tag,
            None => return Err(anyhow!("no notes tag")),
        };
        let coll = Collection::load(tag.hash, self.0.blobs.store()).await?;
        println!("{:#?}", coll);
        for (name, hash) in coll.iter() {
            let data_bytes = self.0.blobs.get_bytes(hash.as_bytes()).await?;
            let text = String::from_utf8(data_bytes.to_vec())?;
            let mut split_name = name
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            // remove the .md
            split_name.truncate(split_name.len() - 3);
            self.create(split_name, text).await?
        }
        Ok(())
    }
    // End direct doc manipulation
}

async fn info_loop(events: impl Stream<Item = Result<LiveEvent>>) {
    warn!("Start info loop");
    tokio::pin!(events);
    loop {
        tokio::select! {
        Some(event) = events.next() => {
            let event = match event {
                Ok(event) => event,
                Err(err) => {
                    error!("{:#?}",err);
                    break;
                },
            };
            warn!("{:#?}",event);
        }}
    }
}
