// This is a local router for this project
// Update trait to include (for Ingress) a request rewriter.

use std::collections::HashMap;

use anyhow::anyhow;
use hyper::Uri;
// use tracing::{debug, warn};
use wash_runtime::{
    engine::workload::ResolvedWorkload,
    host::{
        allowed_hosts::AllowedHost,
        http::{IngressRoute, RouteError, Router},
    },
};
use wasmtime_wasi_http::{RequestOptions, WasiBody};

/// Development router that routes all requests to the last resolved workload
#[derive(Default)]
pub struct EasyRouter {
    //routes:  routing map , per namespace , name
    routes: std::sync::RwLock<HashMap<String, String>>,
}

#[async_trait::async_trait]
impl Router for EasyRouter {
    async fn on_workload_resolved(
        &self,
        resolved_handle: &ResolvedWorkload,
        _component_id: &str,
    ) -> anyhow::Result<()> {
        // debug!(
        //     "ATTACH WORKSPACE {:#?} ==> {}",
        //     &resolved_handle.name(),
        //     component_id
        // );
        let item_name = format!("/{}", &resolved_handle.name());
        let mut lock = self
            .routes
            .write()
            .map_err(|e| anyhow::anyhow!("EasyRouter write lock poisoned: {e}"))?;
        lock.insert(item_name.to_string(), resolved_handle.id().to_string());
        Ok(())
    }

    async fn on_workload_unbind(&self, _workload_id: &str) -> anyhow::Result<()> {
        let mut _ock = self
            .routes
            .write()
            .map_err(|e| anyhow::anyhow!("EasyRouter write lock poisoned: {e}"))?;
        // TODO remove the route lock
        Ok(())
    }

    async fn on_service_http_resolved(
        &self,
        _workload_id: &str,
        _routes: &[IngressRoute],
    ) -> anyhow::Result<()> {
        // need to add these in too.
        Ok(())
    }

    fn allow_outgoing_request(
        &self,
        _workload_id: &str,
        _request: &hyper::Request<WasiBody>,
        _options: Option<RequestOptions>,
        _allowed_hosts: &[AllowedHost],
    ) -> anyhow::Result<()> {
        // check_allowed_hosts(request, allowed_hosts)
        Err(anyhow!("No HTTP access for you!"))
    }

    // Do some rewriting magic on the web request.
    // this is ugly and filled with upwraps.

    fn rewrite_incoming_request(
        &self,
        mut req: hyper::Request<hyper::body::Incoming>,
    ) -> hyper::Request<hyper::body::Incoming> {
        let mut parts = req.uri().clone().into_parts();
        let url_path = req.uri().path();
        // Grab the lock
        let map_lock = self.routes.try_read().unwrap();

        let new_path = match map_lock
            .iter()
            .find(|(key, _)| url_path.starts_with(key.as_str()))
        {
            Some((path, _)) => url_path.trim_start_matches(path),
            None => url_path,
        };

        parts.path_and_query = Some(new_path.parse().unwrap());

        *req.uri_mut() = Uri::from_parts(parts).unwrap();

        req
    }

    /// Pick a workload ID based on the incoming request
    fn route_incoming_request(
        &self,
        req: &hyper::Request<hyper::body::Incoming>,
    ) -> Result<String, RouteError> {
        // println!("{:#?}", req.uri().path());
        let map_lock = self
            .routes
            .try_read()
            .map_err(|_| RouteError::Unavailable)?;
        // println!("the map {:#?}", &map_lock);
        let url_path = req.uri().path();
        let wl = map_lock
            .iter()
            .find(|(key, _)| url_path.starts_with(key.as_str()));
        // println!("{:#?}", &wl);
        match wl {
            Some((_, workload_id)) => Ok(workload_id.clone()),
            None => Err(RouteError::NoWorkloadForHost(String::new())),
        }
    }
}
