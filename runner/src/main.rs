//! A wasm cloud attached to an Iroh node.

use anyhow::Result;
use clap::Parser;
use std::{path::PathBuf, str::FromStr, sync::Arc};

mod custom_plugin;
mod engine;
mod storage;
mod trace;
mod webrouter;
mod workloads;

use custom_plugin::CustomPlugin;
use storage::WasmLoader;
use webrouter::EasyRouter;
// use workloads::WorkloadWrapped;

// The wash stuff
use wash_runtime::{
    engine::Engine, host::{HostApi, HostBuilder, http::Ingress}, plugin::{
        wasi_blobstore::InMemoryBlobstore, wasi_config::DynamicConfig, wasmcloud_secrets::WasmcloudSecrets
    },
};

// Local tracing
use tracing::info;

use crate::workloads::Dispatcher;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to wasm
    input: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    service: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup and start some logging.
    trace::start_logging();

    // Get the args and print them
    let args = Args::parse();
    println!("{:#?}", &args);

    let blobs = storage::Storage::new(PathBuf::from_str("data")?).await?;

    // put a wasm into blob storage
    if let Some(path) = args.input {
        blobs.save(path, args.service).await?;
    }

    // Create the engine with pooling enabled
    let engine = Engine::builder().with_pooling_allocator(true).build()?;
    let http_handler = Ingress::new(EasyRouter::default(), "0.0.0.0:8080".parse()?).await?;
    let config_plugin = DynamicConfig::default();
    let cust_plug = CustomPlugin::default();

    // Blob store thingo
    // let wasm_blobs = MultiplexedAsyncBlobstore::new().with_provider(Arc::new(InMemoryProvider));
    let wasm_blobs = InMemoryBlobstore::new(None);
    let sec_plugin = WasmcloudSecrets::new();
    // Build and start the host
    let host = HostBuilder::new()
        .with_engine(engine)
        .with_friendly_name("my-custom-host")
        .with_http_handler(Arc::new(http_handler))
        .with_plugin(Arc::new(config_plugin))?
        .with_plugin(Arc::new(cust_plug))?
        .with_plugin(Arc::new(wasm_blobs))?
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

    // Load the default items
    let item_list = blobs.list_tags("").await?;
    info!("{:#?}", &item_list);
    for name in item_list.iter() {
        // info!("load me {}", &name);
        if let Some(data) = blobs.get_item(name.clone()).await? {
            let _ = WasmLoader::new(name.to_string(), data.clone())?;

            // info!("have the blob {}",name);
            // Fix the name for the router;
            let short_name = name
                .trim_start_matches("preboot::")
                .trim_end_matches(".wasm");
            let wlr = dispatcher.simple(short_name, data)?;
            println!("{} = {:#?}", name, wlr.workload_id);
            let _wr = host.workload_start(wlr).await?;
            // info!("{:#?}",wr);
        }
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}
