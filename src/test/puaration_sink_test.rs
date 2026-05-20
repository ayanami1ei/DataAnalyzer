// PuarationSink 的单元测试
// 测试纯度统计、复合评分、Top-N 向量排名等功能

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    data::data::Data,
    sink::{
        data_sink::DataSink,
        puaration::{PuarationSink, TopRankedVector},
        quality_inspection::quality_stat::BatchQuality,
    },
};

// 构造一条用于纯度测试的生产数据
// material_code: 物料品名, batch_no: 批号, 其余为工艺参数
fn build_data(
    material_code: &str,
    batch_no: &str,
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

    // 添加各字段到数据对象中
    data.add_pair("物料品名".to_string(), material_code.to_string());
    data.add_pair("批号".to_string(), batch_no.to_string());
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

// 构造一个批次质量统计对象，用于与纯度数据联合测试
// quant_pct / qual_pct: 定量/定性合格百分比
fn make_quality(material: &str, batch: &str, quant_pct: f64, qual_pct: f64) -> BatchQuality {
    BatchQuality {
        batch_no: batch.to_string(),
        material_code: material.to_string(),
        items: vec![],
        quant_total: 10,
        quant_ok: (quant_pct / 100.0 * 10.0).round() as usize,
        qual_total: 10,
        qual_ok: (qual_pct / 100.0 * 10.0).round() as usize,
        quant_isok_pct: quant_pct,
        qual_isok_pct: qual_pct,
        errors: vec![],
    }
}

const PURATION_CONFIG: &str = "config/test/puaration.json";

#[tokio::test]
// 验证：纯度计算以 distinct_vector_count 为分母
// 相同的工艺向量合并统计出现次数，不同物料/向量的纯度按各自 distinct 向量计数计算
async fn sink_should_compute_puaration_with_distinct_vector_count_denominator() {
    let sink = PuarationSink::new(PURATION_CONFIG);

    // P1-V1: 同物料同工艺向量出现2次（含一条重复批号）
    let p1_v1 = Arc::new(Mutex::new(build_data(
        "P1", "B001", "1", "2", "3", "4", "5", "6", "7", "E1",
    )));
    let p1_v1_dup = Arc::new(Mutex::new(build_data(
        "P1", "B002", "1", "2", "3", "4", "5", "6", "7", "E1",
    )));
    // P1-V2: 同物料不同工艺向量出现1次
    let p1_v2 = Arc::new(Mutex::new(build_data(
        "P1", "B003", "1", "2", "3", "9", "5", "6", "7", "E1",
    )));

    // P2-V1: 另一物料出现1次
    let p2_v1 = Arc::new(Mutex::new(build_data(
        "P2", "B004", "11", "12", "13", "14", "15", "16", "17", "E2",
    )));

    // 写入所有数据
    sink.sink(Arc::clone(&p1_v1)).await.expect("sink p1_v1");
    sink.sink(Arc::clone(&p1_v1_dup)).await.expect("sink p1_v1_dup");
    sink.sink(Arc::clone(&p1_v2)).await.expect("sink p1_v2");
    sink.sink(Arc::clone(&p2_v1)).await.expect("sink p2_v1");

    let stats = sink.stats();

    // 验证 P1-V1: 出现2次 / 本物料共2个 distinct 向量 → 纯度 1.0
    let p1_v1_stat = stats
        .iter()
        .find(|x| x.material_code == "P1" && x.process_vector == "1|2|3|4|5|6|7|E1")
        .expect("find P1 V1 stat");
    assert_eq!(p1_v1_stat.occurrence_count, 2);
    assert_eq!(p1_v1_stat.distinct_vector_count, 2);
    assert!((p1_v1_stat.purity - 1.0).abs() < 1e-9);

    // 验证 P1-V2: 出现1次 / 本物料共2个 distinct 向量 → 纯度 0.5
    let p1_v2_stat = stats
        .iter()
        .find(|x| x.material_code == "P1" && x.process_vector == "1|2|3|9|5|6|7|E1")
        .expect("find P1 V2 stat");
    assert_eq!(p1_v2_stat.occurrence_count, 1);
    assert_eq!(p1_v2_stat.distinct_vector_count, 2);
    assert!((p1_v2_stat.purity - 0.5).abs() < 1e-9);

    // 验证 P2-V1: 出现1次 / 本物料共1个 distinct 向量 → 纯度 1.0
    let p2_v1_stat = stats
        .iter()
        .find(|x| x.material_code == "P2" && x.process_vector == "11|12|13|14|15|16|17|E2")
        .expect("find P2 V1 stat");
    assert_eq!(p2_v1_stat.occurrence_count, 1);
    assert_eq!(p2_v1_stat.distinct_vector_count, 1);
    assert!((p2_v1_stat.purity - 1.0).abs() < 1e-9);
}

#[test]
// 验证：复合得分由纯度、定量合格率、定性合格率取三者平均值
fn composite_score_blends_purity_quant_qual() {
    let stat = crate::sink::puaration::puaration_stat::PuarationStat {
        material_code: "M1".to_string(),
        process_vector: "a|b".to_string(),
        occurrence_count: 5,
        distinct_vector_count: 2,
        purity: 1.0,
        quant_isok_pct: 80.0,
        qual_isok_pct: 90.0,
    };
    let score = stat.composite_score();
    let expected = (1.0 + 0.80 + 0.90) / 3.0;
    assert!((score - expected).abs() < 1e-9);
}

#[test]
// 验证：复合得分将纯度上限限制为 1.0，即使原始纯度超过 1.0
fn composite_score_clamps_purity_at_one() {
    let stat = crate::sink::puaration::puaration_stat::PuarationStat {
        material_code: "M1".to_string(),
        process_vector: "a|b".to_string(),
        occurrence_count: 10,
        distinct_vector_count: 1,
        purity: 10.0,
        quant_isok_pct: 50.0,
        qual_isok_pct: 50.0,
    };
    let score = stat.composite_score();
    let expected = (1.0 + 0.50 + 0.50) / 3.0;
    assert!((score - expected).abs() < 1e-9);
}

#[tokio::test]
// 验证：按复合得分排序，每个物料选取 Top-N 工艺向量
async fn top_vectors_per_material_selects_top_n_by_composite() {
    let sink = PuarationSink::new(PURATION_CONFIG);

    // M1: V1 出现2次(高良率), V2 出现3次(低良率) → V1 排名应高于 V2
    // M2: 单一向量
    let batches = vec![
        ("M1", "B001", "1", "2", "3", "4", "5", "6", "7", "E1"),
        ("M1", "B002", "1", "2", "3", "4", "5", "6", "7", "E1"),
        ("M1", "B003", "1", "2", "3", "9", "5", "6", "7", "E1"),
        ("M1", "B004", "1", "2", "3", "9", "5", "6", "7", "E1"),
        ("M1", "B005", "1", "2", "3", "9", "5", "6", "7", "E1"),
        ("M2", "B006", "8", "8", "8", "8", "8", "8", "8", "E2"),
    ];

    for (mat, batch, c, j, id, od, ss, sc, as_, eq) in batches {
        let d = Arc::new(Mutex::new(build_data(
            mat, batch, c, j, id, od, ss, sc, as_, eq,
        )));
        sink.sink(Arc::clone(&d)).await.expect("sink");
    }

    // 给每个批次设置质量数据
    let mut quality = HashMap::new();
    quality.insert(
        "B001".to_string(),
        make_quality("M1", "B001", 90.0, 85.0),
    );
    quality.insert(
        "B002".to_string(),
        make_quality("M1", "B002", 90.0, 85.0),
    );
    quality.insert(
        "B003".to_string(),
        make_quality("M1", "B003", 60.0, 50.0),
    );
    quality.insert(
        "B004".to_string(),
        make_quality("M1", "B004", 60.0, 50.0),
    );
    quality.insert(
        "B005".to_string(),
        make_quality("M1", "B005", 60.0, 50.0),
    );
    quality.insert(
        "B006".to_string(),
        make_quality("M2", "B006", 100.0, 100.0),
    );

    let top = sink.top_vectors_per_material(2, &quality);

    assert!(!top.is_empty(), "should have results");

    // M1: V1 (1|2|3|4|5|6|7|E1) 对应的高良率批次 → rank 1
    //      V2 (1|2|3|9|5|6|7|E1) 对应的低良率批次 → rank 2
    let m1_entries: Vec<&TopRankedVector> =
        top.iter().filter(|v| v.material_code == "M1").collect();
    assert_eq!(m1_entries.len(), 2, "M1 should have 2 top vectors");
    assert_eq!(m1_entries[0].rank, 1);
    assert_eq!(m1_entries[1].rank, 2);

    // M2: 单一向量, rank 1
    let m2_entries: Vec<&TopRankedVector> =
        top.iter().filter(|v| v.material_code == "M2").collect();
    assert_eq!(m2_entries.len(), 1, "M2 should have 1 top vector");
    assert_eq!(m2_entries[0].rank, 1);
    assert_eq!(m2_entries[0].process_vector, "8|8|8|8|8|8|8|E2");
}

#[tokio::test]
// 验证：Top-N 结果严格受限于每个物料的 N 值
async fn top_vectors_respects_n_per_material() {
    let sink = PuarationSink::new(PURATION_CONFIG);

    // 为 M1 生成 3 个不同的工艺向量
    for i in 0..3 {
        let v = format!("{}", i);
        let d = Arc::new(Mutex::new(build_data(
            "M1", &format!("B{}", i), &v, &v, &v, &v, &v, &v, &v, "E1",
        )));
        sink.sink(Arc::clone(&d)).await.expect("sink");
    }

    let quality = HashMap::new();
    // N=2 时只应返回 2 个向量
    let top = sink.top_vectors_per_material(2, &quality);

    let m1: Vec<&TopRankedVector> = top.iter().filter(|v| v.material_code == "M1").collect();
    assert_eq!(m1.len(), 2, "should cap at n=2 per material");
    assert_eq!(m1[0].rank, 1);
    assert_eq!(m1[1].rank, 2);
}

#[tokio::test]
// 验证：无数据时返回空结果
async fn top_vectors_empty_with_no_data() {
    let sink = PuarationSink::new(PURATION_CONFIG);
    let quality = HashMap::new();
    let top = sink.top_vectors_per_material(5, &quality);
    assert!(top.is_empty());
}

#[tokio::test]
// 验证：返回结果按物料代码排序，同物料内按 rank 排序
async fn top_vectors_sorted_by_material_then_rank() {
    let sink = PuarationSink::new(PURATION_CONFIG);

    let d1 = Arc::new(Mutex::new(build_data(
        "B", "B001", "1", "1", "1", "1", "1", "1", "1", "E1",
    )));
    let d2 = Arc::new(Mutex::new(build_data(
        "A", "A001", "2", "2", "2", "2", "2", "2", "2", "E2",
    )));

    sink.sink(Arc::clone(&d1)).await.expect("sink");
    sink.sink(Arc::clone(&d2)).await.expect("sink");

    let quality = HashMap::new();
    let top = sink.top_vectors_per_material(1, &quality);

    // A 应在 B 之前
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].material_code, "A");
    assert_eq!(top[1].material_code, "B");
}
