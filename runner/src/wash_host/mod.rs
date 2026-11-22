// This is the wasmcloud runner host
mod custom_plugin;
mod engine;
mod storage;
mod webrouter;
mod workloads;

use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{Result};
use custom_plugin::CustomPlugin;
use storage::{Storage, WasmLoader};
use tracing::info;
use webrouter::EasyRouter;

// use workloads::WorkloadWrapped;

// The wash stuff
use wash_runtime::{
    engine::Engine,
    host::{Host, HostApi, HostBuilder, http::Ingress},
    plugin::{
        wasi_blobstore::{InMemoryProvider, MultiplexedAsyncBlobstore, MultiplexedBlobstore},
        wasi_config::DynamicConfig,
        // wasi_keyvalue::{InMemoryProvider as MemKeys, MultiplexedKeyValue},
        wasmcloud_secrets::WasmcloudSecrets,
    },
};

use crate::wash_host::workloads::Dispatcher;

pub struct WasmHost {
    host: Arc<Host>,
    storage: Storage,
    dispatcher: Dispatcher,
}

impl WasmHost {
    pub async fn new() -> Result<Self> {
        let store = storage::Storage::new(PathBuf::from_str("data")?).await?;

        // Create the engine with pooling enabled
        let engine = Engine::builder().with_pooling_allocator(true).build()?;
        let http_handler = Ingress::new(EasyRouter::default(), "0.0.0.0:8080".parse()?).await?;
        let config_plugin = DynamicConfig::default();
        let cust_plug = CustomPlugin::default();

        // Blob store thingo
        let wasm_blobs = MultiplexedAsyncBlobstore::new().with_provider(Arc::new(InMemoryProvider));
        let wasm_blobs2 = MultiplexedBlobstore::new(); //.with_provider(Arc::new(InMemoryProvider));
        let wasm_keys = wash_runtime::plugin::wasi_keyvalue::InMemoryKeyValue::new();

        // let wasm_blobs = InMemoryBlobstore::new(None);
        let sec_plugin = WasmcloudSecrets::new();

        // Build and start the host
        let host = HostBuilder::new()
            .with_engine(engine)
            .with_friendly_name("my-custom-host")
            .with_http_handler(Arc::new(http_handler))
            .with_plugin(Arc::new(config_plugin))?
            .with_plugin(Arc::new(cust_plug))?
            .with_plugin(Arc::new(wasm_blobs))?
            .with_plugin(Arc::new(wasm_blobs2))?
            .with_plugin(Arc::new(wasm_keys))?
            .with_plugin(Arc::new(sec_plugin))?
            .build()?;

        // Go
        let host = host.start().await?;
        println!("Host started: {}", host.friendly_name());

        // Make the dispatcher
        let ww = host.wit_world();
        let mut dispatcher = Dispatcher::new(ww);
        // Some test extra's for the dispatcher.
        dispatcher.bind();
        dispatcher.show();

        Ok(Self {
            host: host,
            storage: store,
            dispatcher: dispatcher,
        })
    }

    pub async fn upload(&self, path: PathBuf, service: bool) -> Result<()> {
        self.storage.save(path, service).await?;
        Ok(())
    }

    pub async fn startup(&self) -> Result<()> {
        // Load the default items

        let all = self.storage.list_tags("").await?;

        for item in all.iter() {
            println!("{}", item);
        }

        let item_list = self.storage.list_tags("preboot::").await?;

        info!("{:#?}", &item_list);

        for name in item_list.iter() {
            // info!("load me {}", &name);
            if let Some(data) = self.storage.get_item(name.clone()).await? {
                let _ = WasmLoader::new(name.to_string(), data.clone())?;
                // info!("have the blob {}",name);
                // Fix the name for the router;
                let short_name = name
                    .trim_start_matches("preboot::")
                    .trim_end_matches(".wasm");
                let wlr = self.dispatcher.simple(short_name, data)?;
                println!("{} = {:#?}", name, wlr.workload_id);
                let wr = self.host.workload_start(wlr).await?;
                info!("{:#?}", wr);
            }
        }
        Ok(())
    }
}
