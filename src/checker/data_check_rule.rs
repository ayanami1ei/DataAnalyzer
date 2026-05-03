use crate::{data::data::Data, error::Error};

pub trait DataCheckRule: Send {
    fn check(&self, data: &Data) -> Result<(), Error>;
}
