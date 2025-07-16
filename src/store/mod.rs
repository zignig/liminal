// //! Table definitions and accessors for the redb database.
// use redb::{ReadableTable, TableDefinition, TableError};

// pub(super) const CONFIG: TableDefinition<String, String> = TableDefinition::new("config-0");

// #[derive(Debug, Clone)]
// pub struct Config(Arc<Inner>);


// #[derive(Debug)]
// pub struct Inner { 
//     db: redb::Database
// }


mod fileshow;

pub use fileshow::FileSet;
