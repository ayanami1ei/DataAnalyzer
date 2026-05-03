use crate::{checker::data_check_rule::DataCheckRule, data::data::Data, error::Error};

pub struct GeometryCheckRule;

impl GeometryCheckRule {
    pub fn new() -> Self {
        Self
    }

    fn parse_numeric_by_field(data: &Data, field_name: &str, field_label: &str) -> Result<f64, Error> {
        let key = field_name.to_string();
        let pair = data.get_pair(&key).ok_or_else(|| {
            Error::DataError(format!("{} field '{}' not found in data", field_label, field_name))
        })?;

        pair.value.trim().parse::<f64>().map_err(|_| {
            Error::DataError(format!(
                "{} is not numeric, field '{}', value '{}'",
                field_label, field_name, pair.value
            ))
        })
    }
}

impl DataCheckRule for GeometryCheckRule {
    fn check(&self, data: &Data) -> Result<(), Error> {
        // 规则实现直接声明其所需变量名。
        let inner_die = Self::parse_numeric_by_field(data, "挤出内模(mm)", "inner_die")?;
        let outer_die = Self::parse_numeric_by_field(data, "挤出外模(mm)", "outer_die")?;
        if inner_die >= outer_die {
            return Err(Error::DataError(format!(
                "illegal geometry: inner_die({}) must be less than outer_die({})",
                inner_die, outer_die
            )));
        }

        let core_od = Self::parse_numeric_by_field(data, "缆芯外径（mm)", "core_od")?;
        let jacket_od = Self::parse_numeric_by_field(data, "护套外径(mm)", "jacket_od")?;
        if core_od >= jacket_od {
            return Err(Error::DataError(format!(
                "illegal geometry: core_od({}) must be less than jacket_od({})",
                core_od, jacket_od
            )));
        }

        Ok(())
    }
}
