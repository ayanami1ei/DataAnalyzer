use crate::{
    data::data::Data,
    sink::{
        data_sink::DataSink, end_sink::EndSinkType,
        material_production::MaterialProductionSink,
    },
};

fn build_data(material_code: &str, batch_no: &str) -> Data {
    let mut data = Data::new("material_production_test".to_string());
    data.add_pair("物料品名".to_string(), material_code.to_string());
    data.add_pair("批号".to_string(), batch_no.to_string());
    data
}

#[test]
fn sink_should_count_distinct_batch_by_material() {
    let mut sink = MaterialProductionSink::new(EndSinkType {});

    let mut m1_b1 = build_data("M1", "B1");
    let mut m1_b1_dup = build_data("M1", "B1");
    let mut m1_b2 = build_data("M1", "B2");
    let mut m1_b3_invalid = build_data("M1", "B3");
    m1_b3_invalid.mark_invalid("invalid row".to_string());
    let mut m2_b9 = build_data("M2", "B9");

    sink.sink(&mut m1_b1).expect("sink m1 b1");
    sink.sink(&mut m1_b1_dup).expect("sink m1 b1 dup");
    sink.sink(&mut m1_b2).expect("sink m1 b2");
    sink.sink(&mut m1_b3_invalid).expect("sink m1 b3 invalid");
    sink.sink(&mut m2_b9).expect("sink m2 b9");

    let stats = sink.stats();

    let m1_stat = stats
        .iter()
        .find(|x| x.material_code == "M1")
        .expect("find M1 stat");
    assert_eq!(m1_stat.production_count, 2);

    let m2_stat = stats
        .iter()
        .find(|x| x.material_code == "M2")
        .expect("find M2 stat");
    assert_eq!(m2_stat.production_count, 1);
}
