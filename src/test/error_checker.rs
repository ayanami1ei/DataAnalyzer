use crate::test::log_entry::LogEntry;
use mysql::Row;

pub struct ErrorChecker;

impl ErrorChecker {
    pub fn check(entry: &LogEntry, row: &Row) -> bool {
        match entry.error_type.as_str() {
            "挤出内模大于等于挤出外模" => {
                let inner_die: f64 = row.get("inner_die").unwrap_or(0.0);
                let outer_die: f64 = row.get("outer_die").unwrap_or(0.0);
                inner_die >= outer_die
            }
            e if e.starts_with("找不到") => false,
            _ => false,
        }
    }
}
