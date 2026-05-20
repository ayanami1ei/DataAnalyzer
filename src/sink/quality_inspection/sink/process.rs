use std::collections::HashMap;

use rusqlite::types::Value as SqlValue;

use crate::{
    config, data::data::Data,
    sink::quality_inspection::{
        quality_param,
        quality_stat::*,
    },
};

use super::QualityInspectionSink;

impl QualityInspectionSink {
    /// 处理单条检验数据，解析参数并重新计算 IsOK
    fn process_item(
        data: &Data,
        batch_no: &str,
        material_code: &str,
        quant_total: &mut usize,
        quant_ok: &mut usize,
        qual_total: &mut usize,
        qual_ok: &mut usize,
        errors: &mut Vec<IsokError>,
    ) -> Option<QualityItem> {
        let qi = config::quality_inspection_fields();
        let check_item = Self::extract_str(data, &qi.check_item);
        let param_raw = Self::extract_str(data, &qi.param_raw);
        let test_value = Self::extract_str(data, &qi.test_value);
        let original_isok = Self::extract_bool(data, &qi.isok);

        if param_raw.is_empty() { return None; }

        let parsed = quality_param::parse_param(&param_raw);
        let (param_type_str, lower, upper, correct_isok) = match parsed {
            Ok(pt) => Self::parse_ok_result(pt, &test_value),
            Err(_) => ("未知", None, None, None),
        };

        let correct_isok_val = correct_isok.unwrap_or(original_isok);

        if param_type_str == "定量" {
            *quant_total += 1;
            if correct_isok_val { *quant_ok += 1; }
        } else if param_type_str == "定性" {
            *qual_total += 1;
            if correct_isok_val { *qual_ok += 1; }
        }

        if let Some(correct) = correct_isok {
            if correct != original_isok {
                errors.push(IsokError {
                    batch_no: batch_no.to_string(),
                    material_code: material_code.to_string(),
                    check_item: check_item.clone(),
                    param_raw: param_raw.clone(),
                    test_value: test_value.clone(),
                    expected_isok: correct,
                    actual_isok: original_isok,
                });
            }
        }

        Some(QualityItem {
            batch_no: batch_no.to_string(),
            material_code: material_code.to_string(),
            check_item,
            param_raw,
            param_type: param_type_str.to_string(),
            lower_bound: lower,
            upper_bound: upper,
            test_value,
            original_isok,
            correct_isok: correct_isok_val,
        })
    }

    /// 根据参数类型和检验值计算 IsOK 结果，返回参数类型描述、上下界和重新计算的 IsOK
    fn parse_ok_result(
        pt: quality_param::ParamType,
        test_value: &str,
    ) -> (&'static str, Option<f64>, Option<f64>, Option<bool>) {
        let type_str = if pt.is_quantitative() { "定量" } else { "定性" };
        let (lb, ub) = match &pt {
            quality_param::ParamType::BothBounds { lower, upper } => (Some(*lower), Some(*upper)),
            quality_param::ParamType::LowerBound { lower } => (Some(*lower), None),
            quality_param::ParamType::UpperBound { upper } => (None, Some(*upper)),
            quality_param::ParamType::Qualitative { .. } => (None, None),
        };
        let isok = quality_param::recalc_isok(&pt, test_value).ok();
        (type_str, lb, ub, isok)
    }

    /// 计算百分比，总数为 0 时返回默认值
    fn calc_pct(ok: usize, total: usize) -> f64 {
        let sc = config::score_constants();
        if total > 0 { ok as f64 / total as f64 * 100.0 } else { sc.default_pass_pct }
    }

    /// 将 QualityItem 转换为 SQLite 行值
    fn build_detail_row(item: &QualityItem) -> Vec<SqlValue> {
        vec![
            SqlValue::Text(item.batch_no.clone()),
            SqlValue::Text(item.material_code.clone()),
            SqlValue::Text(item.check_item.clone()),
            SqlValue::Text(item.param_raw.clone()),
            SqlValue::Text(item.param_type.clone()),
            item.lower_bound.map(|v| SqlValue::Real(v)).unwrap_or(SqlValue::Null),
            item.upper_bound.map(|v| SqlValue::Real(v)).unwrap_or(SqlValue::Null),
            SqlValue::Text(item.test_value.clone()),
            SqlValue::Integer(if item.original_isok { 1 } else { 0 }),
            SqlValue::Integer(if item.correct_isok { 1 } else { 0 }),
        ]
    }

    /// 将 IsokError 转换为 SQLite 行值
    fn build_error_row(err: &IsokError) -> Vec<SqlValue> {
        vec![
            SqlValue::Text(err.batch_no.clone()),
            SqlValue::Text(err.material_code.clone()),
            SqlValue::Text(err.check_item.clone()),
            SqlValue::Text(err.param_raw.clone()),
            SqlValue::Text(err.test_value.clone()),
            SqlValue::Integer(if err.expected_isok { 1 } else { 0 }),
            SqlValue::Integer(if err.actual_isok { 1 } else { 0 }),
        ]
    }

    /// 处理所有检验记录，按批号分组，返回批次汇总、明细行和错误行
    pub(super) fn process_records(
        records: &[Data],
    ) -> (Vec<BatchQuality>, Vec<Vec<SqlValue>>, Vec<Vec<SqlValue>>) {
        let qi = config::quality_inspection_fields();
        let mut by_batch: HashMap<String, Vec<&Data>> = HashMap::new();
        for data in records {
            by_batch.entry(Self::extract_str(data, &qi.batch_no)).or_default().push(data);
        }

        let mut batches = Vec::new();
        let mut detail_rows = Vec::new();
        let mut error_rows = Vec::new();

        for (batch_no, data_list) in &by_batch {
            let material_code = Self::extract_material_code(data_list);
            let mut items = Vec::new();
            let mut errors = Vec::new();
            let mut qt = 0usize; let mut qo = 0usize;
            let mut qlt = 0usize; let mut qlo = 0usize;

            for data in data_list {
                if let Some(item) = Self::process_item(data, batch_no, &material_code,
                    &mut qt, &mut qo, &mut qlt, &mut qlo, &mut errors) {
                    items.push(item);
                }
            }

            batches.push(BatchQuality {
                batch_no: batch_no.clone(),
                material_code: material_code.clone(),
                items: items.clone(),
                quant_total: qt, quant_ok: qo,
                qual_total: qlt, qual_ok: qlo,
                quant_isok_pct: Self::calc_pct(qo, qt),
                qual_isok_pct: Self::calc_pct(qlo, qlt),
                errors: errors.clone(),
            });

            detail_rows.extend(items.iter().map(Self::build_detail_row));
            error_rows.extend(errors.iter().map(Self::build_error_row));
        }

        (batches, detail_rows, error_rows)
    }

    /// 从同一批的多个数据中提取物料代码，优先取物料品名，取不到则回退到物料品号
    fn extract_material_code(data_list: &[&Data]) -> String {
        let qi = config::quality_inspection_fields();
        data_list.first().map(|d| {
            let mc = Self::extract_str(d, &qi.material_name);
            if mc.is_empty() { Self::extract_str(d, &qi.material_code) } else { mc }
        }).unwrap_or_default()
    }
}
