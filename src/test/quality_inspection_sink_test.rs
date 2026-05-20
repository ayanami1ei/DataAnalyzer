// QualityInspectionSink 的单元测试
// 测试质检记录的收集、IsOK 异常检测、定量/定性合格率计算、按批次分组

use std::sync::{Arc, Mutex};

use crate::{
    data::data::Data,
    sink::{
        data_sink::DataSink,
        quality_inspection::QualityInspectionSink,
    },
};

// 构造一条质检数据，包含批号、物料、检查项目、检验参数、检验值、IsOK 标记
fn make_quality_data(
    batch_no: &str,
    material: &str,
    check_item: &str,
    param: &str,
    test_value: &str,
    isok: &str,
) -> Data {
    let mut data = Data::new("quality_test".to_string());

    data.add_pair("批号".to_string(), batch_no.to_string());
    data.add_pair("物料品名".to_string(), material.to_string());
    data.add_pair("检查项目".to_string(), check_item.to_string());
    data.add_pair(
        "检验参数(标准参数为数字时无需填写)".to_string(),
        param.to_string(),
    );
    data.add_pair("检验值".to_string(), test_value.to_string());
    data.add_pair("IsOK".to_string(), isok.to_string());

    data
}

#[tokio::test]
// 验证：Sink 能正确收集多条质检记录，并按批次组织到 BatchQuality 中
async fn sink_should_collect_quality_records() {
    let sink = QualityInspectionSink::new("config/test/quality_inspection.json");

    let d1 = Arc::new(Mutex::new(make_quality_data(
        "B001", "M1", "光缆外径", "10.400≤V≤11.200", "10.410", "True",
    )));
    let d2 = Arc::new(Mutex::new(make_quality_data(
        "B001", "M1", "平均厚度", "2.550≤V≤2.800", "2.39", "False",
    )));

    sink.sink(Arc::clone(&d1)).await.expect("sink d1");
    sink.sink(Arc::clone(&d2)).await.expect("sink d2");

    sink.finish().expect("finish should succeed");

    let results = sink.get_results();
    let bq = results.get("B001").expect("should have B001");

    assert_eq!(bq.material_code, "M1");
    assert_eq!(bq.items.len(), 2);
}

#[tokio::test]
// 验证：Sink 能检测到 IsOK 标记与实际数值计算不一致的错误
// 数值在范围内但 IsOK=False → 记录一条 error
async fn sink_should_detect_isok_errors() {
    let sink = QualityInspectionSink::new("config/test/quality_inspection.json");

    // 值在边界范围内但 IsOK=False → 属于异常
    let d = Arc::new(Mutex::new(make_quality_data(
        "B002", "M1", "外径", "1.950≤V≤2.050", "2.000", "False",
    )));

    sink.sink(Arc::clone(&d)).await.expect("sink");
    sink.finish().expect("finish");

    let results = sink.get_results();
    let bq = results.get("B002").expect("should have B002");

    assert_eq!(bq.errors.len(), 1);
    assert_eq!(bq.errors[0].check_item, "外径");
    assert_eq!(bq.errors[0].expected_isok, true);
    assert_eq!(bq.errors[0].actual_isok, false);
}

#[tokio::test]
// 验证：Sink 能正确计算定量项目的合格百分比
async fn sink_should_compute_quant_isok_pct() {
    let sink = QualityInspectionSink::new("config/test/quality_inspection.json");

    // 3 个定量检验项：2 OK + 1 Not OK
    let data_items = vec![
        ("B003", "M1", "item1", "0.100≤V≤1.000", "0.500", "True"),   // OK
        ("B003", "M1", "item2", "0.100≤V≤1.000", "1.500", "False"),  // Not OK
        ("B003", "M1", "item3", "0.100≤V≤1.000", "0.800", "True"),   // OK
    ];

    for (batch, mat, item, param, val, isok) in data_items {
        let d = Arc::new(Mutex::new(make_quality_data(
            batch, mat, item, param, val, isok,
        )));
        sink.sink(Arc::clone(&d)).await.expect("sink");
    }

    sink.finish().expect("finish");

    let results = sink.get_results();
    let bq = results.get("B003").expect("should have B003");

    assert_eq!(bq.quant_total, 3);
    assert_eq!(bq.quant_ok, 2);
    assert!((bq.quant_isok_pct - 66.666666).abs() < 0.001);
}

#[tokio::test]
// 验证：Sink 能正确计算定性项目的合格百分比
async fn sink_should_compute_qual_isok_pct() {
    let sink = QualityInspectionSink::new("config/test/quality_inspection.json");

    // 3 个定性检验项：2 OK + 1 NG
    let data_items = vec![
        ("B004", "M1", "外观", "V=外观应完好", "OK", "True"),
        ("B004", "M1", "松紧度", "V=松紧度应适中", "OK", "True"),
        ("B004", "M1", "渗水", "V=不渗水", "NG", "False"),
    ];

    for (batch, mat, item, param, val, isok) in data_items {
        let d = Arc::new(Mutex::new(make_quality_data(
            batch, mat, item, param, val, isok,
        )));
        sink.sink(Arc::clone(&d)).await.expect("sink");
    }

    sink.finish().expect("finish");

    let results = sink.get_results();
    let bq = results.get("B004").expect("should have B004");

    assert_eq!(bq.qual_total, 3);
    assert_eq!(bq.qual_ok, 2);
    assert!((bq.qual_isok_pct - 66.666666).abs() < 0.001);
}

#[tokio::test]
// 验证：不同批次的数据被正确分组到各自的 BatchQuality 中
async fn sink_should_group_by_batch() {
    let sink = QualityInspectionSink::new("config/test/quality_inspection.json");

    let batches = vec![
        ("B005", "M1", "item_a"),
        ("B005", "M1", "item_b"),
        ("B006", "M2", "item_c"),
    ];

    for (batch, mat, item) in &batches {
        let d = Arc::new(Mutex::new(make_quality_data(
            batch, mat, item, "0.0≤V≤1.0", "0.5", "True",
        )));
        sink.sink(Arc::clone(&d)).await.expect("sink");
    }

    sink.finish().expect("finish");

    let results = sink.get_results();
    // 应包含两个批次
    assert_eq!(results.len(), 2);
    assert!(results.contains_key("B005"));
    assert!(results.contains_key("B006"));
}
