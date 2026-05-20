// 几何校验规则：验证数据中几何尺寸字段的合法性（如内模 < 外模，缆芯外径 < 护套外径）

use crate::{checker::data_check_rule::DataCheckRule, config, data::data::Data, error::Error};

// 几何校验器：实现 DataCheckRule trait，对数据的几何参数进行约束检查
pub struct GeometryCheckRule;

impl GeometryCheckRule {
    // 创建几何校验器实例
    pub fn new() -> Self {
        Self
    }

    // 从数据中按字段名提取并解析为浮点数，field_label 用于错误提示
    fn parse_numeric_by_field(data: &Data, field_name: &str, field_label: &str) -> Result<f64, Error> {
        let key = field_name.to_string();
        // 查找数据中对应字段的键值对
        let pair = data.get_pair(&key).ok_or_else(|| {
            Error::DataError(format!("{} field '{}' not found in data", field_label, field_name))
        })?;
        // 将字段值解析为 f64 浮点数
        pair.value.trim().parse::<f64>().map_err(|_| {
            Error::DataError(format!(
                "{} is not numeric, field '{}', value '{}'",
                field_label, field_name, pair.value
            ))
        })
    }
}

// 几何规则的具体校验逻辑
impl DataCheckRule for GeometryCheckRule {
    fn check(&self, data: &Data) -> Result<(), Error> {
        let fields = config::geometry_check_fields();
        let inner_die = Self::parse_numeric_by_field(data, &fields.inner_die, "inner_die")?;
        let outer_die = Self::parse_numeric_by_field(data, &fields.outer_die, "outer_die")?;
        if inner_die >= outer_die {
            return Err(Error::DataError(format!(
                "illegal geometry: inner_die({}) must be less than outer_die({})",
                inner_die, outer_die
            )));
        }

        let core_od = Self::parse_numeric_by_field(data, &fields.core_od, "core_od")?;
        let jacket_od = Self::parse_numeric_by_field(data, &fields.jacket_od, "jacket_od")?;
        if core_od >= jacket_od {
            return Err(Error::DataError(format!(
                "illegal geometry: core_od({}) must be less than jacket_od({})",
                core_od, jacket_od
            )));
        }

        Ok(())
    }
}
