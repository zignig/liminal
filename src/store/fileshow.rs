// This is an attempt to convert collections into a directory structure

use std::{hash, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use dashmap::DashMap;
use fs_tree::FsTree;
use iroh_blobs::{
    BlobFormat, BlobsProtocol, Hash, format::collection::Collection, hashseq::HashSeq,
    ticket::BlobTicket,
};
use n0_future::StreamExt;
use n0_watcher::Watcher;

#[derive(Debug, Clone)]
pub struct FileSet(Arc<Inner>);

#[derive(Debug, Clone)]
pub struct Inner {
    blobs: BlobsProtocol,
    roots: DashMap<String, Item>,
}

// Internal representation
#[derive(Debug, Clone)]
pub enum Item {
    Unloaded {
        hash: Hash,
    },
    Loaded {
        directories: FsTree,
        links: DashMap<String, Hash>,
        hash: Hash,
    },
}

// Return to the file server
pub enum RenderType {
    File { file_name: String },
    Folder { items: Vec<String> },
}

impl FileSet {
    pub fn new(blobs: BlobsProtocol) -> Self {
        Self(Arc::new(Inner {
            blobs: blobs,
            roots: DashMap::new(),
        }))
    }

    // Searches tags and fills the default dictionary.
    pub async fn fill(&self,prefix: &str) {
        let mut tag_scan = self
            .0
            .blobs
            .store()
            .tags()
            .list_prefix(prefix)
            .await
            .unwrap();
        while let Some(event) = tag_scan.next().await {
            let tag = event.unwrap();
            let tag_name = str::from_utf8(&tag.name.0).unwrap().to_owned();
            if !self.0.roots.contains_key(&tag_name) {
                //let tag_name = tag.name.to_string();
                // println!("{}", &tag_name);
                self.0
                    .roots
                    .insert(tag_name, Item::Unloaded { hash: tag.hash });
            }
        }
    }

    pub async fn del_tags(&self, prefix: &str) -> Result<()> {
        self.0.blobs.store().tags().delete_prefix(prefix).await?;
        Ok(())
    }

    // Hands back the hash the root for building tickets
    pub async fn get_hash(&self, root: String) -> Result<Option<Hash>> {
        if let Some(base) = self.0.roots.get(&root) {
            let hash = match base.value() {
                Item::Unloaded { hash } => hash,
                Item::Loaded {
                    directories: _,
                    links: _,
                    hash,
                } => hash,
            };
            return Ok(Some(*hash));
        };
        Ok(None)
    }

    // Hands back a file or folder from a path request
    pub async fn get(&self, root: String, path: &PathBuf) -> Result<Option<RenderType>> {
        // Do we have the collection key at all ?
        if self.0.roots.contains_key(&root) {
            // Check to see if is already expanded
            let mut the_dir = if let Some(base) = self.0.roots.get(&root) {
                match base.value() {
                    Item::Unloaded { hash: _ } => None,
                    Item::Loaded {
                        directories,
                        links: _,
                        hash: _,
                    } => Some(directories.clone()),
                }
            } else {
                None
            };
            // Not Expanded get a mutable key and fill
            if the_dir.is_none() {
                if let Some(mut base) = self.0.roots.get_mut(&root) {
                    the_dir = match base.value() {
                        Item::Unloaded { hash } => {
                            // println!("{:?}",base.key());
                            // let node = self.0.blobs.endpoint().node_addr().initialized().await;
                            // let ticket = BlobTicket::new(node, *hash, BlobFormat::HashSeq);
                            // println!("{:?}", ticket.to_string());
                            // load the collection and covert to fs
                            let collection =
                                Collection::load(hash.clone(), self.0.blobs.store()).await?;
                            let mut directories = FsTree::new_dir();
                            let links: DashMap<String, Hash> = DashMap::new();
                            for (path, hash) in collection {
                                directories = directories.merge(FsTree::from_path_text(&path));
                                links.insert(path, hash);
                            }
                            *base = Item::Loaded {
                                directories: directories.clone(),
                                links: links.clone(),
                                hash: hash.clone(),
                            };
                            Some(directories)
                        }
                        Item::Loaded {
                            directories,
                            links: _,
                            hash: _,
                        } => Some(directories.clone()),
                    };
                }
            };
            if let Some(dir) = the_dir {
                let val = dir.get(path.clone());
                if let Some(d) = val {
                    match d {
                        FsTree::Regular => {
                            // This should return the actual file data.
                            // needs to get size , mime type ( mapped from file extension)
                            let name = path.file_name().unwrap().display().to_string();
                            return Ok(Some(RenderType::File { file_name: name }));
                        }
                        FsTree::Directory(btree_map) => {
                            for (path, _) in btree_map.iter() {
                                println!("{:?} {:?}", path.display().to_string(), path.extension());
                            }
                            let items = btree_map.keys().map(|f| f.display().to_string()).collect();
                            return Ok(Some(RenderType::Folder { items: items }));
                        }
                        _ => return Ok(None),
                    }
                }
            }
        }
        Ok(None)
    }

    // Hands baack the actual file
    // TODO : unfinished.
    pub async fn get_file(&self, root: String, path: &PathBuf) -> Result<Option<Bytes>> {
        if self.0.roots.contains_key(&root) {
            if let Some(base) = self.0.roots.get(&root) {
                match base.value() {
                    Item::Loaded {
                        directories: _,
                        links,
                        hash: _,
                    } => {
                        if let Some(reference) = links.get(&path.display().to_string()) {
                            let h = reference.value().clone();
                            let data = self.0.blobs.store().get_bytes(h).await?;
                            return Ok(Some(data));
                        }
                    }
                    Item::Unloaded { hash: _ } => return Err(anyhow!("unloaded file")),
                }
            }
        }
        Ok(None)
    }

    pub fn list_roots(&self) -> Vec<String> {
        let mut items: Vec<String> = self.0.roots.iter().map(|k| k.key().to_string()).collect();
        items.sort();
        items.reverse();
        items
    }
}
