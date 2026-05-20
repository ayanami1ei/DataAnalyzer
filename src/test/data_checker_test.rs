// DataChecker 与几何校验规则的单元测试
// 测试 GeometryCheckRule 对模具尺寸和缆芯/护套关系的合理性检查

use crate::{
    data::data::Data,
    sink::{data_checker::DataChecker, data_sink::DataSink},
};
use std::sync::{Arc, Mutex};

const CHECKER_CFG: &str = "config/test/data_checker_geometry.json";

// 构造一条包含挤出模具和线缆尺寸的数据
fn build_ready_data(inner_die: &str, outer_die: &str, core_od: &str, jacket_od: &str) -> Data {
    let mut data = Data::new("test".to_string());
    data.add_pair("挤出内模(mm)".to_string(), "".to_string());
    data.add_pair("挤出外模(mm)".to_string(), "".to_string());
    data.add_pair("缆芯外径（mm)".to_string(), "".to_string());
    data.add_pair("护套外径(mm)".to_string(), "".to_string());

    let _ = data.set_variable_value("挤出内模(mm)", inner_die);
    let _ = data.set_variable_value("挤出外模(mm)", outer_die);
    let _ = data.set_variable_value("缆芯外径（mm)", core_od);
    let _ = data.set_variable_value("护套外径(mm)", jacket_od);
    data
}

#[tokio::test]
async fn sink_should_pass_when_inner_die_not_less_than_outer_die() {
    let checker = DataChecker::new(CHECKER_CFG);
    let data = Arc::new(Mutex::new(build_ready_data("7.50", "7.50", "5.30", "9.90")));

    let result = checker.sink(Arc::clone(&data)).await;
    let data = data.lock().expect("lock data");

    assert!(result.is_ok());
    assert!(!data.is_valid());
    assert!(!data.invalid_reasons().is_empty());
}

#[tokio::test]
async fn sink_should_pass_when_core_od_not_less_than_jacket_od() {
    let checker = DataChecker::new(CHECKER_CFG);
    let data = Arc::new(Mutex::new(build_ready_data(
        "7.50", "14.50", "9.90", "9.90",
    )));

    let result = checker.sink(Arc::clone(&data)).await;
    let data = data.lock().expect("lock data");

    assert!(result.is_ok());
    assert!(!data.is_valid());
    assert!(!data.invalid_reasons().is_empty());
}

#[tokio::test]
async fn sink_should_mark_data_valid_when_all_checks_pass() {
    let checker = DataChecker::new(CHECKER_CFG);
    let data = Arc::new(Mutex::new(build_ready_data(
        "7.50", "14.50", "5.30", "9.90",
    )));

    let result = checker.sink(Arc::clone(&data)).await;
    let data = data.lock().expect("lock data");

    assert!(result.is_ok());
    assert!(data.is_valid());
    assert!(data.invalid_reasons().is_empty());
}

#[tokio::test]
async fn sink_should_treat_zero_point_zero_zero_as_valid_numeric_value() {
    let checker = DataChecker::new(CHECKER_CFG);
    let data = Arc::new(Mutex::new(build_ready_data("0.00", "1.00", "0.00", "1.00")));

    let result = checker.sink(Arc::clone(&data)).await;
    let data = data.lock().expect("lock data");

    assert!(result.is_ok());
    assert!(data.is_valid());
    assert!(data.invalid_reasons().is_empty());
}
