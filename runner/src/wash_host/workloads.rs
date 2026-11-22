use std::collections::HashMap;

// Create and dispatch workloads
// Need to include rcan and storage tricks

// use crate::storage::IrohWorkload;
use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use wash_runtime::{
    types::{Component, LocalResources, Service, Workload, WorkloadStartRequest},
    wit::{WitInterface, WitWorld},
};

// Set up a construct to load and save workloads (not yaml)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadWrapped {
    pub name: String,
}

impl WorkloadWrapped {}

// This is more making stuff
pub struct Dispatcher {
    world: WitWorld,
}

// This needs to be fully rcanned.
impl Dispatcher {
    pub fn new(world: WitWorld) -> Self {
        Dispatcher { world: world }
    }

    pub fn show(&self) {
        let import = self.world.imports.clone();
        let export = self.world.exports.clone();
        println!("IMPORTS");
        for item in import.iter() {
            println!("\t{:#?}", item.to_string())
        }
        println!("EXPORTS");
        for item in export.iter() {
            println!("\t{:#?}", item.to_string())
        }
    }

    pub fn bind(&mut self) {
        // self.world.imports.remove(value)
        // let r = default_blob_iface();
        // self.world.imports.insert(r);
        // let s = default_secrets_iface();
        // self.world.imports.insert(s);
    }

    // given an encoded workload load run and check
    // pub fn from_iroh(&self, _workload: IrohWorkload) -> Result<WorkloadStartRequest> {
    //     Err(anyhow!("Iroh load fail!!"))
    // }

    // given a single wasm , Drop it into the service zone

    #[allow(dead_code)]
    pub fn service(&self, name: &str, data: Bytes) -> Result<WorkloadStartRequest> {
        // just offer up all of the hosts available interface
        let available = self.world.exports.clone().into_iter().collect();

        let res = LocalResources::default();

        // Load the simple component
        let comp = Service {
            bytes: data.into(),
            digest: None,
            local_resources: res,
            max_restarts: 4,
        };

        let req = WorkloadStartRequest {
            workload_id: uuid::Uuid::new_v4().to_string(),
            workload: Workload {
                namespace: "service".to_string(),
                name: name.to_string(),
                annotations: HashMap::new(),
                service: Some(comp),
                components: vec![],
                host_interfaces: available,
                volumes: vec![],
            },
        };
        Ok(req)
    }
    // given a single wasm , Drop it into the testing zone
    pub fn simple(&self, name: &str, data: Bytes) -> Result<WorkloadStartRequest> {
        // just offer up all of the hosts available interface
        let mut available: Vec<WitInterface> = self.world.exports.clone().into_iter().collect();
        let mut import_interfaces: Vec<WitInterface> =
            self.world.imports.clone().into_iter().collect();
        available.append(&mut import_interfaces);

        let res = LocalResources::default();

        // Load the simple component
        let comp = Component {
            name: name.to_string(),
            bytes: data.into(),
            digest: None,
            local_resources: res,
            pool_size: 0,
            ..Default::default()
        };

        let req = WorkloadStartRequest {
            workload_id: uuid::Uuid::new_v4().to_string(),
            workload: Workload {
                namespace: "testing".to_string(),
                name: name.to_string(),
                annotations: HashMap::new(),
                service: None,
                components: vec![comp],
                host_interfaces: available,
                volumes: vec![],
            },
        };
        Ok(req)
    }
}

#[allow(dead_code)]
fn default_blob_iface() -> WitInterface {
    WitInterface {
        namespace: "wasmcloud".to_string(),
        package: "blobstore".to_string(),
        interfaces: [
            "blobstore".to_string(),
            "container".to_string(),
            "types".to_string(),
        ]
        .into_iter()
        .collect(),
        version: Some(semver::Version::parse("0.1.0").unwrap()),
        config: HashMap::from([
            ("backend".to_string(), "filesystem".to_string()),
            ("root".to_string(), "tmp/blobstore".to_string()),
        ]),
        // config: Default::default(),
        name: None,
    }
}

#[allow(dead_code)]
fn default_secrets_iface() -> WitInterface {
    WitInterface {
        namespace: "wasmcloud".to_string(),
        package: "secrets".to_string(),
        interfaces: ["store".to_string(), "reveal".to_string()]
            .into_iter()
            .collect(),
        version: Some(semver::Version::parse("2.1.0").unwrap()),
        config: HashMap::from([
            ("registry-username".to_string(), "wasmcloud".to_string()),
            ("registry-password".to_string(), "wamcloud".to_string()),
        ]),
        // config: Default::default(),
        name: None,
    }
}
