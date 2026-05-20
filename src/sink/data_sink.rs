use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use crate::{data::data::Data, error::Error, router::Router};

#[async_trait]
pub trait DataSink: Send + Sync {
    fn new(config_path: &str) -> Self where Self: Sized;

    async fn sink(&self, data: Arc<Mutex<Data>>) -> Result<(), Error>;

    fn finish(&self) -> Result<(), Error> {
        Ok(())
    }

    fn set_router(&self, _router: Weak<Router>) {}
}
