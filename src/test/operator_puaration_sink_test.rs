// OperatorPuarationSink 的单元测试
// 测试按操作员分组统计工艺纯度（occurrence_count / distinct_vector_count / purity）

use crate::{
    data::data::Data,
    sink::{data_sink::DataSink, operator_puaration::OperatorPuarationSink},
};
use std::sync::{Arc, Mutex};

// 构造一条用于操作员纯度测试的数据，包含备注信息和工艺参数
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

#[tokio::test]
// 验证：同一操作员的相同工艺向量合并统计，不同向量按 distinct 计数计算纯度
async fn sink_should_compute_operator_puaration() {
    let sink = OperatorPuarationSink::new("config/test/operator_puaration.json");

    // 唐清林: 1 条向量（core_od=11, outer_die=14）
    let op2_v1 = Arc::new(Mutex::new(build_data(
        "普缆  主机手：唐清林 跟班：魏文",
        "11",
        "14",
    )));

    // 杜伟: V1 出现 2 次（core_od=1, outer_die=4），V2 出现 1 次（core_od=1, outer_die=9）
    let op1_v1 = Arc::new(Mutex::new(build_data(
        "返工   主机手杜伟   跟班黄刚超",
        "1",
        "4",
    )));
    let op1_v1_dup = Arc::new(Mutex::new(build_data(
        "返工   主机手杜伟   跟班黄刚超",
        "1",
        "4",
    )));
    let op1_v2 = Arc::new(Mutex::new(build_data(
        "返工   主机手杜伟   跟班黄刚超",
        "1",
        "9",
    )));

    sink.sink(Arc::clone(&op1_v1)).await.expect("sink op1_v1");
    sink.sink(Arc::clone(&op1_v1_dup)).await.expect("sink op1_v1_dup");
    sink.sink(Arc::clone(&op1_v2)).await.expect("sink op1_v2");
    sink.sink(Arc::clone(&op2_v1)).await.expect("sink op2_v1");

    let stats = sink.stats();

    // 构建预期的工艺向量（完整字段顺序）
    let vec_1_2_3_4: Vec<String> =
        ["1", "2", "3", "4", "5", "6", "7", "E1"].iter().map(|s| s.to_string()).collect();
    let vec_1_2_3_9: Vec<String> =
        ["1", "2", "3", "9", "5", "6", "7", "E1"].iter().map(|s| s.to_string()).collect();
    let vec_11_2_3_14: Vec<String> =
        ["11", "2", "3", "14", "5", "6", "7", "E1"].iter().map(|s| s.to_string()).collect();

    // 杜伟-V1: 出现2次 / 杜伟共2个 distinct → 纯度 1.0
    let duwei_v1 = stats
        .iter()
        .find(|x| x.operator_name == "杜伟" && x.vector_values == vec_1_2_3_4)
        .expect("find duwei v1");
    assert_eq!(duwei_v1.occurrence_count, 2);
    assert_eq!(duwei_v1.distinct_vector_count, 2);
    assert!((duwei_v1.purity - 1.0).abs() < 1e-9);

    // 杜伟-V2: 出现1次 / 杜伟共2个 distinct → 纯度 0.5
    let duwei_v2 = stats
        .iter()
        .find(|x| x.operator_name == "杜伟" && x.vector_values == vec_1_2_3_9)
        .expect("find duwei v2");
    assert_eq!(duwei_v2.occurrence_count, 1);
    assert_eq!(duwei_v2.distinct_vector_count, 2);
    assert!((duwei_v2.purity - 0.5).abs() < 1e-9);

    // 唐清林-V1: 出现1次 / 唐清林共1个 distinct → 纯度 1.0
    let tangqinglin_v1 = stats
        .iter()
        .find(|x| x.operator_name == "唐清林" && x.vector_values == vec_11_2_3_14)
        .expect("find tangqinglin v1");
    assert_eq!(tangqinglin_v1.occurrence_count, 1);
    assert_eq!(tangqinglin_v1.distinct_vector_count, 1);
    assert!((tangqinglin_v1.purity - 1.0).abs() < 1e-9);
}
