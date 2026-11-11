use std::path::PathBuf;

use anyhow::{Result, anyhow};
use iroh_blobs::store::fs::FsStore;
use n0_future::StreamExt;
use bytes::Bytes;
use serde::{Deserialize, Serialize}; 

#[derive(Debug)]
pub struct Storage {
    blobs: FsStore,
}

impl Storage {
    // Create a new one
    pub async fn new(path: PathBuf) -> Result<Self> {
        let mut full_path = path.clone();
        full_path.push("blobs");

        let blobs = FsStore::load(full_path).await?;
        Ok(Self { blobs })
    }

    // List all the tags that we have under this prefix
    pub async fn list_tags(&self, prefix: &str) -> Result<Vec<String>> {
        let mut tags = self.blobs.tags().list_prefix(prefix).await?;
        let mut tag_list = Vec::new();
        while let Some(tag) = tags.next().await {
            let tag = tag?.name;
            let tag_name = String::from_utf8_lossy(&tag.0).into_owned();
            tag_list.push(tag_name);
        }
        Ok(tag_list)
    }

    // Given a tag get the bytes
    pub async fn get_item(&self, name: String) -> Result<Option<Bytes>> {
        let tag = self.blobs.tags().get(&name).await?;
        let name = name.as_str();
        if let Some(tag) = tag {
            let data =  self.blobs.get_bytes(tag.hash).await?;
            return Ok(Some(data));
        }
        Err(anyhow!("bork on getting {}", name))
    }

    // Take a blog and put it into storage
    pub async fn save(&self, path: PathBuf,is_service: bool) -> Result<()> {
        let abs_path = std::path::absolute(&path)?;
        let t = self.blobs.add_path(&abs_path).await?;

        let mut ps = if let Some(file_name) = path.file_name().and_then(|os_str| os_str.to_str()) {
            format!("preboot::{}", file_name)
        } else {
            format!("preboot::{}", path.display())
        };
        if is_service { 
            ps = format!("service-{}",ps);
        }
        println!("{:#?} {:#?}", &t, &ps);
        self.blobs.tags().set(ps, t.hash_and_format()).await?;
        // Delete the auto tag
        self.blobs.tags().delete(t.name).await?;
        Ok(())
    }
}


// Some help structs
#[derive(Debug,Serialize,Deserialize)]
pub struct IrohWorkload { 
    namespace: String,
    service: SignedPair,
    components: Vec<SignedPair>

}

#[derive(Debug,Serialize,Deserialize)]
struct SignedPair { 
    name: String,
    address: iroh_blobs::Hash,
    // signature: 
}

impl IrohWorkload { 

}



/// Ingest a new wasm
/// 
use wit_component::decode  ; 
use tracing::info;
pub struct WasmLoader;

impl WasmLoader { 
    pub async fn new(name: &String , data: Bytes) -> Result<()> { 
        let info = decode(&data)?;
        let resolve = match info {
            wit_component::DecodedWasm::WitPackage(resolve, _id) => resolve,
            wit_component::DecodedWasm::Component(resolve, _id) => resolve,
        };
        info!("{}",name);
        for (_ , world) in resolve.worlds.iter(){ 
            println!("WORLD : {}",world.name)
        }
        // println!("{:#?}",resolve.);
        Ok(())
    }
}