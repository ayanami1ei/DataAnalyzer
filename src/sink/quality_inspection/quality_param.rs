use crate::{config, error::Error};

/// 检验参数类型
///
/// 定量参数有数值上下界约束，定性参数有描述值匹配约束。
#[derive(Debug, Clone)]
pub enum ParamType {
    /// 双边界定量：下界 ≤ V ≤ 上界
    BothBounds { lower: f64, upper: f64 },
    /// 单下界定量：V ≥ 下界
    LowerBound { lower: f64 },
    /// 单上界定量：V ≤ 上界
    UpperBound { upper: f64 },
    /// 定性：V 必须等于指定描述值
    Qualitative { description: String },
}

impl ParamType {
    /// 是否为定量类型
    pub fn is_quantitative(&self) -> bool {
        matches!(self, ParamType::BothBounds { .. } | ParamType::LowerBound { .. } | ParamType::UpperBound { .. })
    }

    /// 是否为定性类型
    pub fn is_qualitative(&self) -> bool {
        matches!(self, ParamType::Qualitative { .. })
    }
}

/// 解析原始检验参数字符串为结构化 ParamType
///
/// 支持格式（按优先级）：
/// - `"下界≤V≤上界"` → BothBounds
/// - `"V=描述值"` → Qualitative
/// - `"V≥下界"` → LowerBound
/// - `"V≤上界"` → UpperBound
/// - 无法匹配 → 返回 Error
pub fn parse_param(raw: &str) -> Result<ParamType, Error> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(Error::DataError("empty param string".into()));
    }

    if let Some(pos) = s.find("≤V≤") {
        let needle = "≤V≤";
        let lower_str = s[..pos].trim();
        let upper_str = strip_unit_suffix(&s[pos + needle.len()..]);
        let lower = lower_str
            .parse::<f64>()
            .map_err(|e| Error::DataError(format!("parse lower bound '{}': {}", lower_str, e)))?;
        let upper = upper_str
            .parse::<f64>()
            .map_err(|e| Error::DataError(format!("parse upper bound '{}': {}", upper_str, e)))?;
        return Ok(ParamType::BothBounds { lower, upper });
    }

    if let Some(desc) = s.strip_prefix("V=") {
        return Ok(ParamType::Qualitative {
            description: desc.to_string(),
        });
    }

    if let Some(val) = s.strip_prefix("V≥") {
        let v = val
            .trim()
            .parse::<f64>()
            .map_err(|e| Error::DataError(format!("parse lower bound '{}': {}", val, e)))?;
        return Ok(ParamType::LowerBound { lower: v });
    }

    if let Some(val) = s.strip_prefix("V≤") {
        let v = val
            .trim()
            .parse::<f64>()
            .map_err(|e| Error::DataError(format!("parse upper bound '{}': {}", val, e)))?;
        return Ok(ParamType::UpperBound { upper: v });
    }

    Err(Error::DataError(format!(
        "unrecognized param format: '{}'",
        raw
    )))
}

/// 去除数值末尾的单位后缀，如 `"50mm"` → `"50"`
fn strip_unit_suffix(s: &str) -> &str {
    let trimmed = s.trim();
    let end = trimmed
        .rfind(|c: char| c.is_ascii_digit() || c == '.')
        .map(|p| p + 1)
        .unwrap_or(trimmed.len());
    trimmed[..end].trim()
}

/// 根据参数类型和检验值重新计算 IsOK
///
/// - 定量 BothBounds：检验值需在 [lower, upper] 范围内
/// - 定量 LowerBound：检验值 ≥ lower
/// - 定量 UpperBound：检验值 ≤ upper
/// - 定性：检验值必须等于 "OK"
/// - 检验值为空或 "None" 时返回错误
pub fn recalc_isok(param_type: &ParamType, test_value: &str) -> Result<bool, Error> {
    let sc = config::score_constants();
    let tv = test_value.trim();
    if tv.is_empty() || tv == sc.empty_value_indicator {
        return Err(Error::DataError("empty test value".into()));
    }

    match param_type {
        ParamType::BothBounds { lower, upper } => {
            let val = tv
                .parse::<f64>()
                .map_err(|e| Error::DataError(format!("parse test value '{}': {}", tv, e)))?;
            Ok(*lower <= val && val <= *upper)
        }
        ParamType::LowerBound { lower } => {
            let val = tv
                .parse::<f64>()
                .map_err(|e| Error::DataError(format!("parse test value '{}': {}", tv, e)))?;
            Ok(val >= *lower)
        }
        ParamType::UpperBound { upper } => {
            let val = tv
                .parse::<f64>()
                .map_err(|e| Error::DataError(format!("parse test value '{}': {}", tv, e)))?;
            Ok(val <= *upper)
        }
        ParamType::Qualitative { .. } => {
            let sc = config::score_constants();
            Ok(tv == sc.qualitative_pass_value)
        }
    }
}
