use crate::{
    data::data::Data,
    sink::{data_sink::DataSink, end_sink::EndSinkType, puaration::PuarationSink},
};

fn build_data(
    material_code: &str,
    core_od: &str,
    jacket_od: &str,
    inner_die: &str,
    outer_die: &str,
    screw_speed: &str,
    screw_current: &str,
    actual_speed: &str,
    equipment_name: &str,
) -> Data {
    let mut data = Data::new("purity_test".to_string());

    data.add_pair("物料品名".to_string(), material_code.to_string());
    data.add_pair("缆芯外径（mm)".to_string(), core_od.to_string());
    data.add_pair("护套外径(mm)".to_string(), jacket_od.to_string());
    data.add_pair("挤出内模(mm)".to_string(), inner_die.to_string());
    data.add_pair("挤出外模(mm)".to_string(), outer_die.to_string());
    data.add_pair(
        "螺杆速度(rpm)-(挤塑主机速度)（转/分）".to_string(),
        screw_speed.to_string(),
    );
    data.add_pair("螺杆电流（A）".to_string(), screw_current.to_string());
    data.add_pair(
        "实际生产速度（m/min）".to_string(),
        actual_speed.to_string(),
    );
    data.add_pair("设备名称".to_string(), equipment_name.to_string());

    data
}

#[test]
fn sink_should_compute_puaration_with_distinct_vector_count_denominator() {
    let mut sink = PuarationSink::new(EndSinkType {});

    let mut p1_v1 = build_data("P1", "1", "2", "3", "4", "5", "6", "7", "E1");
    let mut p1_v1_dup = build_data("P1", "1", "2", "3", "4", "5", "6", "7", "E1");
    let mut p1_v2 = build_data("P1", "1", "2", "3", "9", "5", "6", "7", "E1");
    let mut p1_invalid = build_data("P1", "8", "2", "3", "4", "5", "6", "7", "E1");
    p1_invalid.mark_invalid("invalid row".to_string());

    let mut p2_v1 = build_data("P2", "11", "12", "13", "14", "15", "16", "17", "E2");

    sink.sink(&mut p1_v1).expect("sink p1_v1");
    sink.sink(&mut p1_v1_dup).expect("sink p1_v1_dup");
    sink.sink(&mut p1_v2).expect("sink p1_v2");
    sink.sink(&mut p1_invalid).expect("sink p1_invalid");
    sink.sink(&mut p2_v1).expect("sink p2_v1");

    let stats = sink.stats();

    let p1_v1_stat = stats
        .iter()
        .find(|x| x.material_code == "P1" && x.process_vector == "1|2|3|4|5|6|7|E1")
        .expect("find P1 V1 stat");
    assert_eq!(p1_v1_stat.occurrence_count, 2);
    assert_eq!(p1_v1_stat.distinct_vector_count, 2);
    assert!((p1_v1_stat.purity - 1.0).abs() < 1e-9);

    let p1_v2_stat = stats
        .iter()
        .find(|x| x.material_code == "P1" && x.process_vector == "1|2|3|9|5|6|7|E1")
        .expect("find P1 V2 stat");
    assert_eq!(p1_v2_stat.occurrence_count, 1);
    assert_eq!(p1_v2_stat.distinct_vector_count, 2);
    assert!((p1_v2_stat.purity - 0.5).abs() < 1e-9);

    let p2_v1_stat = stats
        .iter()
        .find(|x| x.material_code == "P2" && x.process_vector == "11|12|13|14|15|16|17|E2")
        .expect("find P2 V1 stat");
    assert_eq!(p2_v1_stat.occurrence_count, 1);
    assert_eq!(p2_v1_stat.distinct_vector_count, 1);
    assert!((p2_v1_stat.purity - 1.0).abs() < 1e-9);
}
