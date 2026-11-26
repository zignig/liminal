// The turso database that holds the auth info
mod tables;

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use iroh::PublicKey;
use tracing::warn;
use turso::{Builder, Database};

use super::id_comms::Fren;
use tables::ENDPOINTS;
// The persistent store for the auth data
pub struct PersistStore {
    database: Database,
}

impl PersistStore {
    pub async fn new(path: PathBuf) -> Result<Self> {
        warn!("Create database {}", &path.display());
        if let Some(path) = path.to_str() {
            let db = Builder::new_local(path).build().await?;
            let s = Self { database: db };
            s.setup().await?;
            return Ok(s);
        } else {
            return Err(anyhow!("bad db string"));
        }
    }

    pub async fn new_mem() -> Result<Self> {
        warn!("Create database memory");
        let db = Builder::new_local(":memory:").build().await?;
        let s = Self { database: db };
        s.setup().await?;
        Ok(s)
    }

    async fn setup(&self) -> Result<()> {
        let conn = self.database.connect()?;
        conn.execute(ENDPOINTS,()).await?;
        Ok(())
    }

    pub async fn add(&mut self, _pk: PublicKey) -> Result<()> {
        Err(anyhow!("broken add"))
    }

    pub async fn get(&mut self, _pk: &PublicKey) -> Result<Option<Fren>> {
        Err(anyhow!("broken get"))
    }

    pub async fn remove(&mut self, _ph: &PublicKey) -> Result<()> {
        Err(anyhow!("broken remove"))
    }

    pub async fn insert(&mut self, _pk: &PublicKey) -> Result<()> {
        Err(anyhow!("broken insert"))
    }

    pub async fn list(&mut self) -> Result<Vec<Fren>> {
        Err(anyhow!("broken remove"))
    }
}
