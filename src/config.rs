//! Keep authors and documents, base node info in a redb.
//! TODO : unfinished.

use anyhow::{Result, anyhow};
use iroh::{NodeAddr, NodeId, PublicKey, SecretKey};
use redb::{
    Database, DatabaseError, Error, ReadableTable, ReadableTableMetadata, Table, TableDefinition,
    TableHandle, WriteTransaction,
};
use std::path::PathBuf;

const TIME_TABLE: TableDefinition<&str, u64> = TableDefinition::new("timings");
const NODE_TABLE: TableDefinition<&[u8; 32], &str> = TableDefinition::new("nodes");
const SECRET_TABLE: TableDefinition<&[u8; 32], &[u8; 32]> = TableDefinition::new("secrets");

pub struct Info {
    db: Database,
}

pub struct Tables<'tx> {
    pub timing: Table<'tx, &'static str, u64>,
    pub nodes: Table<'tx, &'static [u8; 32], &'static str>,
    pub secrets: Table<'tx, &'static [u8; 32], &'static [u8; 32]>,
}

impl<'tx> Tables<'tx> {
    pub fn new(tx: &'tx WriteTransaction) -> Result<Self, redb::TableError> {
        let timing = tx.open_table(TIME_TABLE)?;
        let nodes = tx.open_table(NODE_TABLE)?;
        let secrets = tx.open_table(SECRET_TABLE)?;
        Ok(Self {
            timing,
            nodes,
            secrets,
        })
    }
}

impl Info {
    pub fn new(name: &PathBuf) -> Result<Self> {
        let db = match Database::create(name) {
            Ok(database) => database,
            Err(_) => return Err(anyhow!("bad database")),
        };
        let write_tx = db.begin_write()?;
        let _ = Tables::new(&write_tx)?;
        write_tx.commit()?;
        let read_tx = db.begin_read()?;
        for i in read_tx.list_tables()? {
            println!("{:?}", i.name());
        }
        Ok(Self { db: db })
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
            println!("{:?}, {:?}", NodeId::from_bytes(k.value()), v.value());
        }
        Ok(())
    }

    pub fn get_secret_key(&self) -> Result<SecretKey> {
        let read_tx = self.db.begin_read()?;
        let secrets = read_tx.open_table(SECRET_TABLE)?;
        if let Some((key, _)) = secrets.first()? {
            return Ok(SecretKey::from_bytes(key.value()));
        } else {
            println!("Make a new secret");
            let write_tx = self.db.begin_write()?;
            let secret = SecretKey::generate(rand::rngs::OsRng);
            {
                let mut secrets = write_tx.open_table(SECRET_TABLE)?;
                secrets.insert(&secret.to_bytes(), &secret.to_bytes())?;
            }
            write_tx.commit()?;
            return Ok(secret);
        }
    }
}
