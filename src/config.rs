//! Keep authors and documents, base node info in a redb.
//! TODO : unfinished.

use std::path::PathBuf;

use redb::{Database, Error, ReadableTable, TableDefinition};

const TABLE: TableDefinition<&str, u64> = TableDefinition::new("my_data");


pub struct info{
    db: Database
}

impl info { 
    pub fn new(name : &PathBuf){
            let db = Database::open(name);
    }

}
