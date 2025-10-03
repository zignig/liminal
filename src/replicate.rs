// Make a replicator using the iroh-smol-kv
//

use std::time::Duration;

use iroh::{PublicKey, SecretKey};
use iroh_blobs::BlobsProtocol;
use iroh_gossip::{net::Gossip, proto::TopicId};
use iroh_smol_kv::{
    Config,
    api::{self, Client, Filter, Subscribe, SubscribeResult},
    util::format_bytes,
};
use n0_future::StreamExt;
use n0_snafu::{Result, ResultExt};
use tokio::task;

pub struct Replicator {
    blobs: BlobsProtocol,
    client: Client,
    secret: SecretKey,
}

impl Replicator {
    pub async fn new(
        gossip: Gossip,
        blobs: BlobsProtocol,
        topic_id: TopicId,
        bootstrap: Vec<PublicKey>,
        secret: SecretKey,
    ) -> Result<Self> {
        let topic = gossip.subscribe(topic_id, bootstrap).await.e()?;
        let client = Client::local(topic, Config::default());
        Ok(Self {
            blobs,
            client,
            secret,
        })
    }

    // Testing for the kv share.
    pub async fn run(&self) -> Result<()> {
        let client = self.client.clone();
        let secret = self.secret.clone();
        let blobs = self.blobs.clone();
        task::spawn(test_runner(client, secret, blobs));
        Ok(())
    }
}

// Add to the kv once an hour, do it first...
pub async fn test_runner(client: Client, secret: SecretKey, blobs: BlobsProtocol) -> Result<()> {
    let ws = client.write(secret);
    let mut op_id = 0;
    let mut next_op_id = || {
        let id = op_id;
        op_id += 1;
        id
    };
    loop {
        let id = next_op_id();
        println!("update count {:?}", id);
        // println!("boop");
        let mut tag_scan = blobs.store().tags().list_prefix("col").await.unwrap();
        while let Some(event) = tag_scan.next().await {
            let tag = event.unwrap();
            let tag_name = str::from_utf8(&tag.name.0).unwrap().to_owned();
            let _ = ws.put(tag_name, tag.hash.to_hex()).await;
        }
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
