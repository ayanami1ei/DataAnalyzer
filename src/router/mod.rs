use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use tracing::warn;

use crate::{
    data::data::Data,
    error::Error,
    router::route::Route,
    sink::{data_sink::DataSink, regist_info::RegistInfo},
};

pub mod route;
pub mod toml_reader;

inventory::collect!(RegistInfo);

pub struct Router {
    route_map: HashMap<BTreeSet<String>, Arc<dyn DataSink>>,
    sink_map: HashMap<String, Arc<dyn DataSink>>,
}

impl Router {
    pub fn new(config_path: &str) -> Result<Self, Error> {
        let toml_text = std::fs::read_to_string(config_path)?;
        let config: TomlRouterConfig = toml::from_str(&toml_text).map_err(|e| {
            Error::SourceDataFileError(format!("router config parse failed: {}", e))
        })?;
        Self::from_routes(config.routes, config.sinks.unwrap_or_default())
    }

    fn from_routes(
        routes: Vec<Route>,
        sink_configs: HashMap<String, String>,
    ) -> Result<Self, Error> {
        let mut sink_map = HashMap::new();
        for entry in inventory::iter::<RegistInfo> {
            if let Some(cfg_path) = sink_configs.get(entry.name) {
                let sink = (entry.constructor)(cfg_path);
                sink_map.insert(entry.name.to_string(), Arc::from(sink));
            }
        }

        let missing: Vec<&str> = routes
            .iter()
            .filter(|r| !sink_map.contains_key(r.to.as_str()))
            .map(|r| r.to.as_str())
            .collect();
        if !missing.is_empty() {
            return Err(Error::SourceDataFileError(format!(
                "sinks not configured in [sinks]: {:?}",
                missing
            )));
        }

        let mut route_map = HashMap::new();
        for route in &routes {
            let sink = sink_map
                .get(route.to.as_str())
                .ok_or_else(|| {
                    Error::SourceDataFileError(format!(
                        "router sink [{}] not registered",
                        route.to
                    ))
                })?;
            route_map.insert(route.when.clone(), Arc::clone(sink));
        }

        Ok(Self { route_map, sink_map })
    }

    pub fn attach_router(router: &Arc<Self>) {
        let weak = Arc::downgrade(router);
        for (_, sink) in &router.sink_map {
            sink.set_router(weak.clone());
        }
    }

    pub fn get_sink(&self, name: &str) -> Option<Arc<dyn DataSink>> {
        self.sink_map.get(name).cloned()
    }

    pub fn finish_all(&self) {
        for (name, sink) in &self.sink_map {
            if let Err(e) = sink.finish() {
                warn!("router: sink '{}' finish failed: {}", name, e);
            }
        }
    }

    pub async fn route_data(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        let state_set = data
            .lock()
            .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?
            .flow_state_set();
        if let Some(sink) = self.route_map.get(&state_set) {
            sink.sink(data).await
        } else {
            warn!("router: no sink matched state set {:?}", state_set);
            Ok(())
        }
    }
}

#[async_trait]
impl DataSink for Router {
    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error> {
        self.route_data(data).await
    }

    fn new(_config_path: &str) -> Self {
        unimplemented!("Router is not created via DataSink::new")
    }
}

#[derive(Deserialize)]
struct TomlRouterConfig {
    routes: Vec<Route>,
    sinks: Option<HashMap<String, String>>,
}
