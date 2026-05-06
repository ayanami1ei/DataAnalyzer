use crate::{
    data::data::Data,
    sink::{
        data_sink::DataSink, database::sqlite_database_sink::SqliteDatabaseSink,
        end_sink::EndSinkType,
    },
};

fn build_ready_data(batch_no: &str, material_name: &str) -> Data {
    let mut data = Data::new("test".to_string());
    data.add_pair("批号".to_string(), "".to_string());
    data.add_pair("物料品名".to_string(), "".to_string());
    data.add_pair("实际生产速度（m/min）".to_string(), "".to_string());
    data.add_pair("缆芯外径（mm)".to_string(), "".to_string());
    data.add_pair("挤出内模(mm)".to_string(), "".to_string());
    data.add_pair("挤出外模(mm)".to_string(), "".to_string());
    data.add_pair("护套外径(mm)".to_string(), "".to_string());
    data.add_pair(
        "螺杆速度(rpm)-(挤塑主机速度)（转/分）".to_string(),
        "".to_string(),
    );
    data.add_pair("螺杆电流（A）".to_string(), "".to_string());
    data.add_pair("生产速度（米/分）".to_string(), "".to_string());

    let _ = data.set_variable_value("批号", batch_no);
    let _ = data.set_variable_value("物料品名", material_name);
    let _ = data.set_variable_value("实际生产速度（m/min）", "10.0");
    let _ = data.set_variable_value("缆芯外径（mm)", "5.0");
    let _ = data.set_variable_value("挤出内模(mm)", "6.0");
    let _ = data.set_variable_value("挤出外模(mm)", "10.0");
    let _ = data.set_variable_value("护套外径(mm)", "8.0");
    let _ = data.set_variable_value("螺杆速度(rpm)-(挤塑主机速度)（转/分）", "20.0");
    let _ = data.set_variable_value("螺杆电流（A）", "30.0");
    let _ = data.set_variable_value("生产速度（米/分）", "40.0");
    data
}

#[test]
fn sqlite_sink_should_split_valid_and_invalid_rows() {
    let cfg_path = "target/sqlite_sink_test_config.json";
    let db_path = "target/sqlite_sink_test.db";

    let _ = std::fs::remove_file(db_path);
    let cfg_text = format!(
        "{{\n  \"sqlite_db_path\": \"{}\",\n  \"table_name\": \"work_record\",\n  \"columns\": [\n    \"批号\",\n    \"物料品名\",\n    \"实际生产速度（m/min）\",\n    \"缆芯外径（mm)\",\n    \"挤出内模(mm)\",\n    \"挤出外模(mm)\",\n    \"护套外径(mm)\",\n    \"螺杆速度(rpm)-(挤塑主机速度)（转/分）\",\n    \"螺杆电流（A）\",\n    \"生产速度（米/分）\"\n  ],\n  \"buffer_capacity\": 4,\n  \"flush_threshold\": 2,\n  \"progress_print_every\": 100\n}}",
        db_path
    );
    std::fs::write(cfg_path, cfg_text).expect("write sqlite test config");

    {
        let end = Box::new(EndSinkType {});
        let mut sink = SqliteDatabaseSink::new(end, cfg_path).expect("new sqlite sink");

        let mut d1 = build_ready_data("B1", "M1");
        let mut d2 = build_ready_data("B2", "M2");
        d2.mark_invalid("test invalid reason".to_string());

        sink.sink(&mut d1).expect("sink d1");
        sink.sink(&mut d2).expect("sink d2");
    }

    let conn = rusqlite::Connection::open(db_path).expect("open sqlite db");
    let mut valid_stmt = conn
        .prepare("SELECT COUNT(*) FROM work_record")
        .expect("prepare valid count sql");
    let valid_count: i64 = valid_stmt
        .query_row([], |row| row.get(0))
        .expect("query valid count");

    let mut invalid_stmt = conn
        .prepare("SELECT COUNT(*) FROM work_record_invalid")
        .expect("prepare invalid count sql");
    let invalid_count: i64 = invalid_stmt
        .query_row([], |row| row.get(0))
        .expect("query invalid count");

    assert_eq!(valid_count, 1);
    assert_eq!(invalid_count, 1);
}
