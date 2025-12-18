//! Keep authors and documents, base node info in a redb.
//! Stores secret key and some peers for now.
//! The layout for this is stolen from the persistant store
//! in iroh-blobs. Seemed like a good layout.

use anyhow::{Result, anyhow};
use iroh::{EndpointId, PublicKey, SecretKey};
use redb::{Database, ReadableDatabase, Table, TableDefinition, WriteTransaction};
use std::{fs, path::PathBuf, str::FromStr};

const TIME_TABLE: TableDefinition<&str, u64> = TableDefinition::new("timings");
const NODE_TABLE: TableDefinition<&[u8; 32], &str> = TableDefinition::new("nodes");
const SECRET_TABLE: TableDefinition<u32, &[u8; 32]> = TableDefinition::new("secrets");
const DOCS_TABLE: TableDefinition<&str, &[u8; 32]> = TableDefinition::new("doc_pointers");
const AUTHORS_TABLE: TableDefinition<&str, &[u8; 32]> = TableDefinition::new("authors");

pub struct Info {
    db: Database,
    current: CurrentTransaction,
}

#[derive(Default)]
enum CurrentTransaction {
    #[default]
    None,
    Read,
    Write,
}

pub struct Tables<'tx> {
    pub timing: Table<'tx, &'static str, u64>,
    pub nodes: Table<'tx, &'static [u8; 32], &'static str>,
    pub secrets: Table<'tx, u32, &'static [u8; 32]>,
    pub docs: Table<'tx, &'static str, &'static [u8; 32]>,
    pub authors: Table<'tx, &'static str, &'static [u8; 32]>,
}

impl<'tx> Tables<'tx> {
    pub fn new(tx: &'tx WriteTransaction) -> Result<Self, redb::TableError> {
        let timing = tx.open_table(TIME_TABLE)?;
        let nodes = tx.open_table(NODE_TABLE)?;
        let secrets = tx.open_table(SECRET_TABLE)?;
        let docs = tx.open_table(DOCS_TABLE)?;
        let authors = tx.open_table(AUTHORS_TABLE)?;

        Ok(Self {
            timing,
            nodes,
            secrets,
            docs,
            authors,
        })
    }
}

impl Info {
    pub fn new(name: &PathBuf) -> Result<Self> {
        fs::create_dir_all(name.parent().unwrap())?;
        let db = match Database::create(name) {
            Ok(database) => database,
            Err(e) => return Err(anyhow!("bad database create, {}", e)),
        };
        let write_tx = db.begin_write()?;
        let _ = Tables::new(&write_tx)?;
        write_tx.commit()?;
        // let read_tx = db.begin_read()?;
        // for i in read_tx.list_tables()? {
        //     println!("{:?}", i.name());
        // }
        Ok(Self {
            db: db,
            current: CurrentTransaction::default(),
        })
    }

    pub fn add_node(&mut self, node: PublicKey) -> Result<()> {
        let write_tx = self.db.begin_write()?;
        {
            let mut nodes = write_tx.open_table(NODE_TABLE)?;
            let _ = nodes.insert(node.as_bytes(), "fren")?;
        }
        write_tx.commit()?;
        Ok(())
    }

    pub fn list_nodes(&self) -> Result<()> {
        let read_tx = self.db.begin_read()?;
        let nodes = read_tx.open_table(NODE_TABLE)?;
        for (k, v) in nodes.range::<&'static [u8; 32]>(..)?.flatten() {
            println!("{:?}, {:?}", EndpointId::from_bytes(k.value()), v.value());
        }
        Ok(())
    }

    pub fn get_secret_key(&self) -> Result<SecretKey> {
        let read_tx = self.db.begin_read()?;
        let secrets = read_tx.open_table(SECRET_TABLE)?;
        if let Some(data) = secrets.get(0)? {
            return Ok(SecretKey::from_bytes(data.value()));
        } else {
            println!("Make a new secret key");
            let write_tx = self.db.begin_write()?;
            let secret = SecretKey::generate(&mut rand::rng());
            {
                let mut secrets = write_tx.open_table(SECRET_TABLE)?;
                secrets.insert(0, &secret.to_bytes())?;
            }
            write_tx.commit()?;
            return Ok(secret);
        }
    }

    pub fn rocket_key(&self) -> Result<[u8; 32]> {
        let read_tx = self.db.begin_read()?;
        let secrets = read_tx.open_table(SECRET_TABLE)?;
        if let Some(data) = secrets.get(1)? {
            return Ok(data.value().clone());
        } else {
            println!("Create secret cookie key");
            let write_tx = self.db.begin_write()?;
            let secret = SecretKey::generate(&mut rand::rng());
            {
                let mut secrets = write_tx.open_table(SECRET_TABLE)?;
                secrets.insert(1, &secret.to_bytes())?;
            }
            write_tx.commit()?;
            return Ok(secret.to_bytes());
        }
    }

    pub fn get_docs_key(&self, name: &str) -> Result<[u8; 32]> {
        let read_tx = self.db.begin_read()?;
        let docs = read_tx.open_table(DOCS_TABLE)?;
        if let Some(data) = docs.get(name)? {
            return Ok(data.value().clone());
        } else {
            return Err(anyhow!("key does not exist"));
        }
    }

    pub fn set_docs_key(&self, name: &str, value: [u8; 32]) -> Result<()> {
        let write_tx = self.db.begin_write()?;
        {
            let mut docs = write_tx.open_table(DOCS_TABLE)?;
            docs.insert(name, &value)?;
        }
        write_tx.commit()?;
        Ok(())
    }

    pub fn get_notes_id(&self) -> Result<[u8; 32]> {
        // let key = iroh_docs::NamespacePublicKey::from_str(
        //     "7c348d28ea5cbe4001bdb21fe9446b6f936b5424a3c1c9712fdda706d1c40181",
        // )?;
        // let _ = self.set_docs_key("notes", *key.as_bytes());
        self.get_docs_key("notes")
    }

    pub fn get_author_key(&self, name: &str) -> Result<[u8; 32]> {
        let read_tx = self.db.begin_read()?;
        let authors = read_tx.open_table(AUTHORS_TABLE)?;
        if let Some(data) = authors.get(name)? {
            return Ok(data.value().clone());
        } else {
            return Err(anyhow!("key does not exist"));
        }
    }

    pub fn set_author_key(&self, name: &str, value: [u8; 32]) -> Result<()> {
        let write_tx = self.db.begin_write()?;
        {
            let mut authors = write_tx.open_table(AUTHORS_TABLE)?;
            authors.insert(name, &value)?;
        }
        write_tx.commit()?;
        Ok(())
    }

    pub fn get_notes_author(&self) -> Result<[u8; 32]> {
        self.get_author_key("notes")
    }
}
