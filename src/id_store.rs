// A redb backed actor to handle Endpoint ids

use std::collections::BTreeMap;

use iroh::EndpointId;
use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};
use postcard::{from_bytes, to_stdvec};
use redb::{Database, ReadableDatabase, ReadableTable, Table, TableDefinition, TypeName, Value, WriteTransaction};
use rocket::{data::N, futures::sink::With};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

// Stored endpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Seen,
    Known,
    Apparent,
    Fren,
    Enemy,
    DestroyOnSight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fren {
    name: String,
    id: EndpointId,
    status: Status,
    created: i64,
}

impl Fren {
    fn new(id: EndpointId) -> Self {
        Self {
            name: "_".to_string(),
            id: id,
            status: Status::Seen,
            created: chrono::Utc::now().timestamp_nanos_opt().expect("time does not exist")
        }
    }
}
// KV impl
impl Value for Fren {
    type SelfType<'a>
        = Fren
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        postcard::from_bytes(data).unwrap()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        to_stdvec(value).unwrap()
    }

    fn type_name() -> redb::TypeName {
        TypeName::new("Fren")
    }
}

// Database
const NODE_TABLE: TableDefinition<&[u8; 32], Fren> = TableDefinition::new("nodes");

// Irpc

#[derive(Debug, Serialize, Deserialize)]
struct Get {
    key: EndpointId,
}

#[derive(Debug, Serialize, Deserialize)]
struct Set {
    key: EndpointId,
    value: Fren,
}

#[derive(Debug, Serialize, Deserialize)]
struct List;


impl From<(EndpointId, Fren)> for Set {
    fn from((key, value): (EndpointId, Fren)) -> Self {
        Self { key, value }
    }
}

#[rpc_requests(message = IdentityMessage, no_rpc, no_spans)]
#[derive(Serialize, Deserialize, Debug)]
enum StorageProtocol {
    #[rpc(tx=oneshot::Sender<Option<Fren>>)]
    Get(Get),
    #[rpc(tx=oneshot::Sender<()>)]
    Set(Set),
    #[rpc(tx=oneshot::Sender<Vec<Fren>>)]
    List(List),
}

struct Actor {
    recv: tokio::sync::mpsc::Receiver<IdentityMessage>,
    db: Database,
}

impl Actor {
    async fn run(mut self) {
        while let Some(msg) = self.recv.recv().await {
            self.handle(msg).await;
        }
    }

    async fn handle(&mut self, msg: IdentityMessage) {
        match msg {
            IdentityMessage::Get(get) => {
                let WithChannels { tx, inner, .. } = get;
                let read_txn = self.db.begin_read().unwrap();
                let table = read_txn.open_table(NODE_TABLE).unwrap();
                let key = inner.key.as_array().expect("bad key");
                let value = match table.get(key).unwrap() {
                    Some(value) => Some(value.value()),
                    None => None,
                };
                tx.send(value).await.ok();
            }

            IdentityMessage::Set(set) => {
                let WithChannels { tx, inner, .. } = set;
                let write_txn = self.db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(NODE_TABLE).unwrap();
                    let key = inner.key.as_array().expect("bad key");
                    table.insert(key, inner.value).unwrap();
                }
                write_txn.commit().unwrap();

                tx.send(()).await.ok();
            }

            IdentityMessage::List(list) => {
                let WithChannels{ tx , .. } = list;
                let read_txn = self.db.begin_read().unwrap();
                let table = read_txn.open_table(NODE_TABLE).unwrap();
                let mut res = Vec::new();
                for item in  table.iter().unwrap(){
                    let (_,item) = item.unwrap();
                    res.push(item.value());
                }
                tx.send(res).await.ok();
            }
        }
    }
}

pub struct IdentityApi {
    tx: Sender<IdentityMessage>,
}

impl IdentityApi {
    pub fn spawn(file_name: &str) -> IdentityApi {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        //Create the database
        let db = Database::create(file_name).unwrap();
        let write_txn = db.begin_write().unwrap();
        let _ = write_txn.open_table(NODE_TABLE).unwrap();
        write_txn.commit().unwrap();

        let actor = Actor { recv: rx, db: db };
        n0_future::task::spawn(actor.run());

        IdentityApi { tx: tx.clone() }
    }

    pub fn client(&self) -> IdClient {
        let tx = self.tx.clone();
        IdClient {
            inner: Client::local(tx),
        }
    }
}

pub struct IdClient {
    inner: Client<StorageProtocol>,
}

impl IdClient {
    pub async fn get(&self, key: EndpointId) -> irpc::Result<Option<Fren>> {
        info!("get {} ", key);
        self.inner.rpc(Get { key }).await
    }

    pub async fn new_fren(&self, key: EndpointId) {
        match self.inner.rpc(Get { key }).await.unwrap() {
            Some(fren) => { 
                warn!("Fren {:#?}",fren);
                return
            },
            None => {
                let value = Fren::new(key);
                self.inner.rpc(Set { key, value }).await.unwrap();
            }
        }
    }

    pub async fn set(&self, key: EndpointId, value: Fren) -> irpc::Result<()> {
        self.inner.rpc(Set { key, value }).await
    }

    pub async fn is_fren(&self, key: EndpointId) -> bool {
        let _val = self.inner.rpc(Get { key }).await;
        false
    }

    pub async fn list(&self) -> irpc::Result<Vec<Fren>> { 
        self.inner.rpc(List {}).await
    }
}
