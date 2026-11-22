// Attempt at custom wit interface and plugin

use std::collections::HashSet;

use tracing::warn;
use wash_runtime::{
    engine::workload::WorkloadItem,
    plugin::{HostPlugin, WitInterfaces},
    wit::{WitInterface, WitWorld},
};

mod bindings {
    wasmtime::component::bindgen!({
        world: "control",
        imports: { default: async | trappable | tracing },
    });
}

// use bindings::Control::;
#[derive(Default)]
pub struct CustomPlugin;


#[async_trait::async_trait]
impl HostPlugin for CustomPlugin {
    fn id(&self) -> &'static str {
        "my_custom_plugin"
    }

    fn world(&self) -> WitWorld {
        WitWorld {
            imports: HashSet::from([WitInterface::from("liminal:controller@0.3.0")]),
            exports: HashSet::from([WitInterface::from("liminal:controller@0.3.0")]),
        }
    }

    async fn on_workload_item_bind<'a>(
        &self,
        component_handle: &mut WorkloadItem<'a>,
        interfaces: WitInterfaces<'_>,
    ) -> anyhow::Result<()> { 
        warn!("LIMINAL INTERFACE");
        warn!("{:#?}", &component_handle);
        warn!("{:#?}", &interfaces);
        Ok(())
    }
    
}
