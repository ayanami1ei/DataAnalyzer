use crate::{data::data::Data, data_creater::WorkRecordCreater, traits::DataCreater};

fn build_template_from_json() -> Data {
    let text = r#"
    {
      "name": "work_record",
      "project_name_column": "项目名称",
      "project_value_column": "项目记录结果",
                        "index_key_set": ["批号", "物料品号", "创建日期"],
            "object_key_set": ["创建日期", "设备名称", "备注信息"],
      "variable_key_set": ["实际生产速度（m/min）"]
    }
    "#;
    let mut data: Data = serde_json::from_str(text).expect("parse template json");
    data.reset_runtime_state();
    data
}

#[test]
fn create_by_batch_should_fill_all_object_keys_from_config() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create by batch");

    let data = creater.get_data().expect("get data");

    assert_eq!(
        data.get_pair(&"创建日期".to_string())
            .expect("创建日期 value")
            .value,
        "2026-04-05"
    );
    assert_eq!(
        data.get_pair(&"设备名称".to_string())
            .expect("设备名称 value")
            .value,
        "EQUIP-A"
    );
    assert_eq!(
        data.get_pair(&"备注信息".to_string())
            .expect("备注信息 value")
            .value,
        "note-1"
    );
}

#[test]
fn create_by_batch_should_keep_previous_object_key_when_later_row_is_blank() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create by batch first row");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "".to_string(),
            "".to_string(),
            "实际生产速度（m/min）".to_string(),
            "68.0".to_string(),
        ])
        .expect("create by batch second row");

    let data = creater.get_data().expect("get data");

    assert_eq!(
        data.get_pair(&"创建日期".to_string())
            .expect("创建日期 value")
            .value,
        "2026-04-05"
    );
    assert_eq!(
        data.get_pair(&"设备名称".to_string())
            .expect("设备名称 value")
            .value,
        "EQUIP-A"
    );
    assert_eq!(
        data.get_pair(&"备注信息".to_string())
            .expect("备注信息 value")
            .value,
        "note-1"
    );
}

#[test]
fn create_by_batch_should_allow_missing_optional_object_key_column() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create by batch should succeed when 备注信息 column is missing");

    let data = creater.get_data().expect("get data");

    assert_eq!(
        data.get_pair(&"创建日期".to_string())
            .expect("创建日期 value")
            .value,
        "2026-04-05"
    );
    assert_eq!(
        data.get_pair(&"设备名称".to_string())
            .expect("设备名称 value")
            .value,
        "EQUIP-A"
    );
}

#[test]
fn create_by_batch_should_match_object_columns_with_bom_and_spaces() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            " 批号 ".to_string(),
            "物料品名".to_string(),
            " 创建日期 ".to_string(),
            "设备名称".to_string(),
            "\u{feff}备注信息 ".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-with-bom-header".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create by batch");

    let data = creater.get_data().expect("get data");
    assert_eq!(
        data.get_pair(&"备注信息".to_string())
            .expect("备注信息 value")
            .value,
        "note-with-bom-header"
    );
}

#[test]
fn create_by_batch_should_convert_excel_serial_date_for_date_key() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "45943".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create by batch");

    let data = creater.get_data().expect("get data");
    assert_eq!(
        data.get_pair(&"创建日期".to_string())
            .expect("创建日期 value")
            .value,
        "2025-10-13"
    );
}

#[test]
fn create_by_batch_should_group_by_configured_index_keys() {
    let text = r#"
    {
      "name": "work_record",
      "project_name_column": "项目名称",
      "project_value_column": "项目记录结果",
      "index_key_set": ["设备名称", "批号"],
      "object_key_set": ["物料品名", "创建日期", "备注信息"],
      "variable_key_set": ["实际生产速度（m/min）"]
    }
    "#;
    let mut template: Data = serde_json::from_str(text).expect("parse template json");
    template.reset_runtime_state();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create row 1");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M002".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-B".to_string(),
            "note-2".to_string(),
            "实际生产速度（m/min）".to_string(),
            "67.0".to_string(),
        ])
        .expect("create row 2");

    let all = creater.drain_all_data().expect("drain all data");
    assert_eq!(all.len(), 2);
}

#[test]
fn create_by_batch_should_split_records_when_same_batch_and_material_but_different_create_date() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create row 1");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-06".to_string(),
            "EQUIP-A".to_string(),
            "note-2".to_string(),
            "实际生产速度（m/min）".to_string(),
            "67.0".to_string(),
        ])
        .expect("create row 2");

    let all = creater.drain_all_data().expect("drain all data");
    assert_eq!(all.len(), 2);
}

#[test]
fn create_by_batch_should_split_records_when_same_batch_and_date_but_different_material() {
    let template = build_template_from_json();
    let mut creater = WorkRecordCreater::new(template);

    creater
        .set_row_elements(vec![
            "批号".to_string(),
            "物料品名".to_string(),
            "创建日期".to_string(),
            "设备名称".to_string(),
            "备注信息".to_string(),
            "项目名称".to_string(),
            "项目记录结果".to_string(),
        ])
        .expect("set row elements");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M001".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-1".to_string(),
            "实际生产速度（m/min）".to_string(),
            "66.0".to_string(),
        ])
        .expect("create row 1");

    creater
        .create_by_batch(vec![
            "B001".to_string(),
            "M002".to_string(),
            "2026-04-05".to_string(),
            "EQUIP-A".to_string(),
            "note-2".to_string(),
            "实际生产速度（m/min）".to_string(),
            "67.0".to_string(),
        ])
        .expect("create row 2");

    let all = creater.drain_all_data().expect("drain all data");
    assert_eq!(all.len(), 2);
}
