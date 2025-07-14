use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use std::{
    path::{Component, Path, PathBuf},
    string,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    name: String,
    children: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Item {
    File { name: String, hash: Hash },
    UnFolder { name: String, hash: Hash },
    Folder { name: String, inner: Folder },
}

impl Folder {
    fn new(name: String) -> Self {
        Self {
            name: name,
            children: vec![],
        }
    }
}


struct FolderMeta { 
    header: [u8;14], // "zignigFolder0"
}
 
impl Folder { 
        pub const HEADER: &'static [u8; 14] = b"zignigFolder0.";

        
}