use crate::test::geometry_error_entry::GeometryErrorEntry;
use mysql::Row;

pub struct LogCheckValidator;

impl LogCheckValidator {
    fn get_numeric_value(row: &Row, column: &str) -> Option<f64> {
        if let Some(value) = row.get::<f64, _>(column) {
            return Some(value);
        }
        row.get::<String, _>(column)
            .and_then(|x| x.trim().parse::<f64>().ok())
    }

    pub fn validate_geometry_error(entry: &GeometryErrorEntry, row: &Row) -> Result<bool, String> {
        // 根据日志中的几何错误类型，验证数据库中的对应字段是否确实违反规则。
        if entry.error_detail.contains("inner_die(") && entry.error_detail.contains("outer_die(") {
            let inner_die = Self::get_numeric_value(row, "挤出内模(mm)")
                .ok_or_else(|| "missing column 挤出内模(mm)".to_string())?;
            let outer_die = Self::get_numeric_value(row, "挤出外模(mm)")
                .ok_or_else(|| "missing column 挤出外模(mm)".to_string())?;
            return Ok(inner_die >= outer_die);
        }

        if entry.error_detail.contains("core_od(") && entry.error_detail.contains("jacket_od(") {
            let core_od = Self::get_numeric_value(row, "缆芯外径（mm)")
                .ok_or_else(|| "missing column 缆芯外径（mm)".to_string())?;
            let jacket_od = Self::get_numeric_value(row, "护套外径(mm)")
                .ok_or_else(|| "missing column 护套外径(mm)".to_string())?;
            return Ok(core_od >= jacket_od);
        }

        if entry.error_detail.contains("is not numeric") {
            let marker = "field '";
            let field_start = entry
                .error_detail
                .find(marker)
                .map(|x| x + marker.len())
                .ok_or_else(|| "unable to locate field marker".to_string())?;
            let field_end = entry.error_detail[field_start..]
                .find('\'')
                .map(|x| x + field_start)
                .ok_or_else(|| "unable to locate field name end".to_string())?;
            let field_name = &entry.error_detail[field_start..field_end];
            return Ok(Self::get_numeric_value(row, field_name).is_none());
        }

        Err(format!(
            "unsupported geometry error detail: {}",
            entry.error_detail
        ))
    }
}
