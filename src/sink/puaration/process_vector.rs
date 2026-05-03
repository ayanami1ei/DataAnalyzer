#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessVector {
    pub values: Vec<String>,
}

impl ProcessVector {
    pub fn signature(&self) -> String {
        self.values.join("|")
    }
}
