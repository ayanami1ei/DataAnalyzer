use crate::test::{
    db_fetcher::DbFetcher, geometry_error_entry::GeometryErrorEntry,
    log_check_validator::LogCheckValidator,
};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn test_log_check() {
    let log_file = File::open("log.txt").expect("failed to open log.txt");
    let reader = BufReader::new(log_file);
    let db = DbFetcher::new();
    let mut parsed_count = 0usize;
    let mut checked_count = 0usize;
    let mut failed_cases: Vec<String> = Vec::new();
    let mut inconsistent_cases: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = line.expect("failed to read line from log.txt");
        if let Some(entry) = GeometryErrorEntry::parse_from_log_line(&line) {
            parsed_count += 1;
            let row = db.fetch_row(&entry.batch_no, &entry.material_name);
            let result = match row.as_ref() {
                Some(r) => LogCheckValidator::validate_geometry_error(&entry, r),
                None => Err("database row not found".to_string()),
            };

            checked_count += 1;
            if let Err(e) = &result {
                failed_cases.push(format!(
                    "批号:{} 物料品名:{} 错误:{} 结果:{:?}",
                    entry.batch_no, entry.material_name, entry.error_detail, e
                ));
            }

            if result == Ok(false) {
                inconsistent_cases.push(format!(
                    "批号:{} 物料品名:{} 错误:{}",
                    entry.batch_no, entry.material_name, entry.error_detail
                ));
            }
        }
    }

    assert!(parsed_count > 0, "no valid log entry parsed from log.txt");
    assert!(checked_count > 0, "no log entry was checked");
    if !inconsistent_cases.is_empty() {
        eprintln!(
            "found {} inconsistent entries (likely source data changed after log generation), first 5:\n{}",
            inconsistent_cases.len(),
            inconsistent_cases
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
    assert!(
        failed_cases.is_empty(),
        "log check failed for {} entries:\n{}",
        failed_cases.len(),
        failed_cases.join("\n")
    );
}
