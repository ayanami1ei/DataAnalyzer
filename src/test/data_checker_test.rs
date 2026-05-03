use crate::{
    checker::geometry_check_rule::GeometryCheckRule,
    data::data::Data,
    sink::{data_checker::DataChecker, data_sink::DataSink, end_sink::EndSinkType},
};

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

#[test]
fn sink_should_pass_when_inner_die_not_less_than_outer_die() {
    let mut checker = DataChecker::new(EndSinkType {}, vec![Box::new(GeometryCheckRule::new())]);
    let mut data = build_ready_data("7.50", "7.50", "5.30", "9.90");

    let result = checker.sink(&mut data);

    assert!(result.is_ok());
    assert!(!data.is_valid());
    assert!(!data.invalid_reasons().is_empty());
}

#[test]
fn sink_should_pass_when_core_od_not_less_than_jacket_od() {
    let mut checker = DataChecker::new(EndSinkType {}, vec![Box::new(GeometryCheckRule::new())]);
    let mut data = build_ready_data("7.50", "14.50", "9.90", "9.90");

    let result = checker.sink(&mut data);

    assert!(result.is_ok());
    assert!(!data.is_valid());
    assert!(!data.invalid_reasons().is_empty());
}

#[test]
fn sink_should_mark_data_valid_when_all_checks_pass() {
    let mut checker = DataChecker::new(EndSinkType {}, vec![Box::new(GeometryCheckRule::new())]);
    let mut data = build_ready_data("7.50", "14.50", "5.30", "9.90");

    let result = checker.sink(&mut data);

    assert!(result.is_ok());
    assert!(data.is_valid());
    assert!(data.invalid_reasons().is_empty());
}

#[test]
fn sink_should_treat_zero_point_zero_zero_as_valid_numeric_value() {
    let mut checker = DataChecker::new(EndSinkType {}, vec![Box::new(GeometryCheckRule::new())]);
    let mut data = build_ready_data("0.00", "1.00", "0.00", "1.00");

    let result = checker.sink(&mut data);

    assert!(result.is_ok());
    assert!(data.is_valid());
    assert!(data.invalid_reasons().is_empty());
}
