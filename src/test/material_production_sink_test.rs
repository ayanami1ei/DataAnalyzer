// MaterialProductionSink 的单元测试
// 测试按物料统计去重批次数量的功能

use std::sync::{Arc, Mutex};

use crate::{
    data::data::Data,
    sink::{data_sink::DataSink, material_production::MaterialProductionSink},
};

// 构造一条物料生产数据，包含物料品名和批号
fn build_data(material_code: &str, batch_no: &str) -> Data {
    let mut data = Data::new("material_production_test".to_string());
    data.add_pair("物料品名".to_string(), material_code.to_string());
    data.add_pair("批号".to_string(), batch_no.to_string());
    data
}

#[tokio::test]
// 验证：Sink 能正确统计每个物料的不同批号个数，重复批号不重复计数
async fn sink_should_count_distinct_batch_by_material() {
    let sink = MaterialProductionSink::new("config/test/material_production.json");

    // M1: B1 出现两次（应计 1）、B2 出现一次 → 总 2 个批次
    // M2: B9 出现一次 → 总 1 个批次
    let m1_b1 = Arc::new(Mutex::new(build_data("M1", "B1")));
    let m1_b1_dup = Arc::new(Mutex::new(build_data("M1", "B1")));
    let m1_b2 = Arc::new(Mutex::new(build_data("M1", "B2")));
    let m2_b9 = Arc::new(Mutex::new(build_data("M2", "B9")));

    sink.sink(Arc::clone(&m1_b1)).await.expect("sink m1 b1");
    sink.sink(Arc::clone(&m1_b1_dup)).await.expect("sink m1 b1 dup");
    sink.sink(Arc::clone(&m1_b2)).await.expect("sink m1 b2");
    sink.sink(Arc::clone(&m2_b9)).await.expect("sink m2 b9");

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
