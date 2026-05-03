use std::collections::HashMap;

pub trait DatabaseRowMapping {
    fn table_name(&self) -> &str;
    fn column_index_value_map(&self) -> HashMap<String, String>;
    fn check_legal(&mut self);
}
