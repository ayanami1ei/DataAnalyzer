use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPair{
    pub key:String,

    #[serde(skip)]
    pub value:String,
    #[serde(skip)]
    pub ready:bool,
}