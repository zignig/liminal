use anyhow::Result;
use iroh::{Endpoint, Watcher, protocol::Router};

pub use self::fren::FrenApi;
pub const FREN_ALPN: &[u8] = b"liminal/fren/0";

mod fren {
    //! Implementation of our storage service.
    //!
    //! The only `pub` item is [`FrenApi`], everything else is private.

    use std::collections::BTreeMap;

    use anyhow::{Context, Result};
    use iroh::{Endpoint, protocol::ProtocolHandler};
    use irpc::{
        Client, WithChannels,
        channel::{mpsc, oneshot},
        rpc::RemoteService,
        rpc_requests,
    };
    // Import the macro
    use irpc_iroh::{IrohProtocol, IrohRemoteConnection};
    use serde::{Deserialize, Serialize};
    use tracing::info;

    #[derive(Debug, Serialize, Deserialize)]
    struct Get {
        key: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct List;

    #[derive(Debug, Serialize, Deserialize)]
    struct Set {
        key: String,
        value: String,
    }

    // Use the macro to generate both the FrenProtocol and FrenMessage enums
    // plus implement Channels for each type
    #[rpc_requests(message = FrenMessage)]
    #[derive(Serialize, Deserialize, Debug)]
    enum FrenProtocol {
        #[rpc(tx=oneshot::Sender<Option<String>>)]
        Get(Get),
        #[rpc(tx=oneshot::Sender<()>)]
        Set(Set),
        #[rpc(tx=mpsc::Sender<String>)]
        List(List),
    }

    struct FrenActor {
        recv: tokio::sync::mpsc::Receiver<FrenMessage>,
        state: BTreeMap<String, String>,
    }

    impl FrenActor {
        pub fn spawn() -> FrenApi {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let actor = Self {
                recv: rx,
                state: BTreeMap::new(),
            };
            n0_future::task::spawn(actor.run());
            FrenApi {
                inner: Client::local(tx),
            }
        }

        async fn run(mut self) {
            while let Some(msg) = self.recv.recv().await {
                self.handle(msg).await;
            }
        }

        async fn handle(&mut self, msg: FrenMessage) {
            match msg {
                FrenMessage::Get(get) => {
                    info!("get {:?}", get);
                    let WithChannels { tx, inner, .. } = get;
                    tx.send(self.state.get(&inner.key).cloned()).await.ok();
                }
                FrenMessage::Set(set) => {
                    info!("set {:?}", set);
                    let WithChannels { tx, inner, .. } = set;
                    self.state.insert(inner.key, inner.value);
                    tx.send(()).await.ok();
                }
                FrenMessage::List(list) => {
                    info!("list {:?}", list);
                    let WithChannels { tx, .. } = list;
                    for (key, value) in &self.state {
                        if tx.send(format!("{key}={value}")).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct FrenApi {
        inner: Client<FrenProtocol>,
    }

    impl FrenApi {
        // pub const FREN_ALPN: &[u8] = b"liminal/fren/0";

        pub fn spawn() -> Self {
            FrenActor::spawn()
        }

        pub fn connect(endpoint: Endpoint, addr: impl Into<iroh::EndpointId>) -> Result<FrenApi> {
            let conn = IrohRemoteConnection::new(endpoint);
            
            Ok(FrenApi {
                inner: Client::boxed(conn),
            })
        }


        pub fn expose(&self) -> Result<impl ProtocolHandler> {
            let local = self
                .inner
                .as_local()
                .context("can not listen on remote service")?;
            Ok(IrohProtocol::new(FrenProtocol::remote_handler(local)))
        }

        pub async fn get(&self, key: String) -> irpc::Result<Option<String>> {
            self.inner.rpc(Get { key }).await
        }

        pub async fn list(&self) -> irpc::Result<mpsc::Receiver<String>> {
            self.inner.server_streaming(List, 10).await
        }

        pub async fn set(&self, key: String, value: String) -> irpc::Result<()> {
            let msg = Set { key, value };
            self.inner.rpc(msg).await
        }
    }
}
