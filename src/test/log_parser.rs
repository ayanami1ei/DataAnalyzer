use crate::test::log_entry::LogEntry;

pub struct LogParser;

impl LogParser {
    pub fn parse_line(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.split('，').collect();
        if parts.len() < 3 {
            return None;
        }
        let batch_no = parts[0].trim_start_matches("批号：").trim().to_string();
        let material_name = parts[1].trim_start_matches("物料品号：").trim().to_string();
        let error_type = parts[2].trim().to_string();
        Some(LogEntry {
            batch_no,
            material_name,
            error_type,
        })
    }
}
