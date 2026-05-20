use serde::Deserialize;
use std::collections::HashMap;

use crate::router::{route::Route, Router};

#[derive(Deserialize)]
struct RouterConfig {
    routes: Vec<Route>,
    sinks: Option<HashMap<String, String>>,
}

pub struct TomlReader;

impl TomlReader {
    pub fn read(config_path: &str) -> Result<Router, crate::error::Error> {
        let toml_text = std::fs::read_to_string(config_path)?;
        let config: RouterConfig = toml::from_str(&toml_text).map_err(|e| {
            crate::error::Error::SourceDataFileError(format!(
                "toml reader parse failed: {}",
                e
            ))
        })?;
        Router::from_routes(config.routes, config.sinks.unwrap_or_default())
    }
}
