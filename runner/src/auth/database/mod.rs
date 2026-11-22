// The turso database that holds the auth info

use turso::database;

// The persistent store for the auth data
pub struct PersistStore;

impl PersistStore {
    pub async new(path: PathBuf) -> Result<Self> { 
        warn!("Create database {}",&path);
    }

    pub async new_mem() -> Result<Self> { 
        warn!("Create database memory",&path);
    }

    
}