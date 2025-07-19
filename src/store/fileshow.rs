// This is an attempt to convert collections into a directory structure

use std::{path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use dashmap::DashMap;
use fs_tree::FsTree;
use iroh_blobs::{Hash, format::collection::Collection, net_protocol::Blobs};
use n0_future::StreamExt;

#[derive(Debug, Clone)]
pub struct FileSet(Arc<Inner>);

#[derive(Debug, Clone)]
pub struct Inner {
    blobs: Blobs,
    roots: DashMap<String, Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Unloaded {
        hash: Hash,
    },
    Loaded {
        directories: FsTree,
        links: DashMap<String, Hash>, 
    },
}

impl FileSet {
    pub fn new(blobs: Blobs) -> Self {
        Self(Arc::new(Inner {
            blobs: blobs,
            roots: DashMap::new(),
        }))
    }

    pub async fn fill(&self) {
        let mut tag_scan = self
            .0
            .blobs
            .store()
            .tags()
            .list_prefix("col")
            .await
            .unwrap();
        while let Some(event) = tag_scan.next().await {
            let tag = event.unwrap();
            let tag_name = str::from_utf8(&tag.name.0).unwrap().to_owned();
            if !self.0.roots.contains_key(&tag_name) {
                //let tag_name = tag.name.to_string();
                println!("{}", &tag_name);
                self.0
                    .roots
                    .insert(tag_name, Item::Unloaded { hash: tag.hash });
            }
        }
    }

    pub async fn get(&self, root: String, path: &PathBuf) -> anyhow::Result<Option<Vec<String>>> {
        if self.0.roots.contains_key(&root) {
            if let Some(mut base) = self.0.roots.get_mut(&root) {
                let the_dir = match base.value() {
                    Item::Unloaded { hash } => {
                        // load the collection and covert to fs
                        let collection =
                            Collection::load(hash.clone(), self.0.blobs.store()).await?;
                        let mut dir = FsTree::new_dir();
                        let links: DashMap<String, Hash> = DashMap::new();
                        for (path, hash) in collection {
                            // println!("{:?}", path);
                            dir = dir.merge(FsTree::from_path_text(&path));
                            links.insert(path, hash);
                        }
                        *base = Item::Loaded {
                            directories: dir.clone(),
                            links: links,
                        };
                        dir
                    }
                    Item::Loaded {
                        directories,
                        links: _,
                    } => {
                        // println!("it's already loaded ! ");
                        directories.clone()
                        // println!("{:#?}",directories.children())
                        // get the path and update the map
                        // self.0.roots.insert(base.key(), )
                    }
                };
                if let Some(dir) = the_dir.get(path.clone()) {
                    // println!("{}", path.display().to_string());
                    // println!("{:#?}", dir.children());
                    if let Some(d) = dir.children() {
                        let r: Vec<String> = d.keys().map(|f| f.display().to_string()).collect();
                        // println!("{:?}", r);
                        return Ok(Some(r));
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn get_file(&self, root: String, path: &PathBuf) -> anyhow::Result<Bytes> {
        if self.0.roots.contains_key(&root) {
            if let Some(mut base) = self.0.roots.get_mut(&root) {
                match base.value() {
                    Item::Loaded {
                        directories: _,
                        links,
                    } => {
                        if let Some(reference) = links.get(&path.display().to_string()) {
                            let h = reference.value().clone();
                            // let data = self.0.blobs.store().get_bytes(knf).await?;
                            let data = self.0.blobs.store().get_bytes(h).await?;
                            return Ok(data);
                        }
                    }
                    Item::Unloaded { hash: _ } => return Err(anyhow!("should already be loaded")),
                }
            }
        }
        Err(anyhow!("no  key!"))
    }

    pub fn list_roots(&self) -> Vec<String> {
        self.0.roots.iter().map(|k| k.key().to_string()).collect()
        // let mut items: Vec<String> = Vec::new();
        // for a in self.0.roots.iter() {
        //     items.push(a.key().to_string());
        // }
        // items
    }
}
