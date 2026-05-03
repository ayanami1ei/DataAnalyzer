use crate::{
    data::data::Data,
    sink::{
        data_sink::DataSink, end_sink::EndSinkType, operator_puaration::OperatorPuarationSink,
    },
};

fn build_data(remark: &str, core_od: &str, outer_die: &str) -> Data {
    let mut data = Data::new("operator_puaration_test".to_string());
    data.add_pair("备注信息".to_string(), remark.to_string());
    data.add_pair("缆芯外径（mm)".to_string(), core_od.to_string());
    data.add_pair("护套外径(mm)".to_string(), "2".to_string());
    data.add_pair("挤出内模(mm)".to_string(), "3".to_string());
    data.add_pair("挤出外模(mm)".to_string(), outer_die.to_string());
    data.add_pair(
        "螺杆速度(rpm)-(挤塑主机速度)（转/分）".to_string(),
        "5".to_string(),
    );
    data.add_pair("螺杆电流（A）".to_string(), "6".to_string());
    data.add_pair("实际生产速度（m/min）".to_string(), "7".to_string());
    data.add_pair("设备名称".to_string(), "E1".to_string());
    data
}

#[test]
fn sink_should_compute_operator_puaration() {
    let mut sink = OperatorPuarationSink::new(EndSinkType {});

    let mut op1_v1 = build_data("返工   主机手杜伟   跟班黄刚超", "1", "4");
    let mut op1_v1_dup = build_data("返工   主机手杜伟   跟班黄刚超", "1", "4");
    let mut op1_v2 = build_data("返工   主机手杜伟   跟班黄刚超", "1", "9");
    let mut op1_invalid = build_data("返工   主机手杜伟   跟班黄刚超", "8", "4");
    op1_invalid.mark_invalid("invalid row".to_string());
    let mut op2_v1 = build_data("普缆  主机手：唐清林 跟班：魏文", "11", "14");

    sink.sink(&mut op1_v1).expect("sink op1_v1");
    sink.sink(&mut op1_v1_dup).expect("sink op1_v1_dup");
    sink.sink(&mut op1_v2).expect("sink op1_v2");
    sink.sink(&mut op1_invalid).expect("sink op1_invalid");
    sink.sink(&mut op2_v1).expect("sink op2_v1");

    let stats = sink.stats();

    let duwei_v1 = stats
        .iter()
        .find(|x| x.operator_name == "杜伟" && x.process_vector == "1|2|3|4|5|6|7|E1")
        .expect("find duwei v1");
    assert_eq!(duwei_v1.occurrence_count, 2);
    assert_eq!(duwei_v1.distinct_vector_count, 2);
    assert!((duwei_v1.purity - 1.0).abs() < 1e-9);

    let duwei_v2 = stats
        .iter()
        .find(|x| x.operator_name == "杜伟" && x.process_vector == "1|2|3|9|5|6|7|E1")
        .expect("find duwei v2");
    assert_eq!(duwei_v2.occurrence_count, 1);
    assert_eq!(duwei_v2.distinct_vector_count, 2);
    assert!((duwei_v2.purity - 0.5).abs() < 1e-9);

    let tangqinglin_v1 = stats
        .iter()
        .find(|x| x.operator_name == "唐清林" && x.process_vector == "11|2|3|14|5|6|7|E1")
        .expect("find tangqinglin v1");
    assert_eq!(tangqinglin_v1.occurrence_count, 1);
    assert_eq!(tangqinglin_v1.distinct_vector_count, 1);
    assert!((tangqinglin_v1.purity - 1.0).abs() < 1e-9);
}
