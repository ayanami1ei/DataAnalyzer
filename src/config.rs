use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

fn load_json<T: for<'a> Deserialize<'a>>(path: &str) -> Option<T> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

// ====================================================================
// 中文业务字段名配置
// ====================================================================

#[derive(Deserialize, Clone)]
pub struct GeometryCheckFields {
    pub inner_die: String,
    pub outer_die: String,
    pub core_od: String,
    pub jacket_od: String,
}

impl Default for GeometryCheckFields {
    fn default() -> Self {
        Self {
            inner_die: "挤出内模(mm)".into(),
            outer_die: "挤出外模(mm)".into(),
            core_od: "缆芯外径（mm)".into(),
            jacket_od: "护套外径(mm)".into(),
        }
    }
}

static GEOMETRY_FIELDS: OnceLock<GeometryCheckFields> = OnceLock::new();

pub fn geometry_check_fields() -> &'static GeometryCheckFields {
    GEOMETRY_FIELDS
        .get_or_init(|| load_json("config/fields/geometry_check_fields.json").unwrap_or_default())
}

#[derive(Deserialize, Clone)]
pub struct KeyAliases {
    #[serde(flatten)]
    pub aliases: HashMap<String, Vec<String>>,
}

impl Default for KeyAliases {
    fn default() -> Self {
        let mut m = HashMap::new();
        m.insert("物料键".into(), vec!["物料品号".into(), "物料品名".into()]);
        m.insert("物料品号".into(), vec!["物料品号".into(), "物料品名".into()]);
        m.insert("物料品名".into(), vec!["物料品名".into(), "物料品号".into()]);
        m.insert("创建日期".into(), vec!["创建日期".into(), "生产日期".into()]);
        Self { aliases: m }
    }
}

static KEY_ALIASES: OnceLock<KeyAliases> = OnceLock::new();

pub fn key_aliases() -> &'static HashMap<String, Vec<String>> {
    &KEY_ALIASES
        .get_or_init(|| load_json("config/fields/key_aliases.json").unwrap_or_default())
        .aliases
}

#[derive(Deserialize, Clone)]
pub struct QualityInspectionFields {
    pub check_item: String,
    pub param_raw: String,
    pub test_value: String,
    pub isok: String,
    pub batch_no: String,
    pub material_name: String,
    pub material_code: String,
    pub detail_columns: Vec<String>,
    pub error_columns: Vec<String>,
}

impl Default for QualityInspectionFields {
    fn default() -> Self {
        Self {
            check_item: "检查项目".into(),
            param_raw: "检验参数(标准参数为数字时无需填写)".into(),
            test_value: "检验值".into(),
            isok: "IsOK".into(),
            batch_no: "批号".into(),
            material_name: "物料品名".into(),
            material_code: "物料品号".into(),
            detail_columns: vec![
                "批号".into(), "物料品名".into(), "检查项目".into(),
                "检验参数".into(), "参数类型".into(), "下界".into(),
                "上界".into(), "检验值".into(), "原始IsOK".into(), "正确IsOK".into(),
            ],
            error_columns: vec![
                "批号".into(), "物料品名".into(), "检查项目".into(),
                "检验参数".into(), "检验值".into(), "期望IsOK".into(), "实际IsOK".into(),
            ],
        }
    }
}

static QUALITY_FIELDS: OnceLock<QualityInspectionFields> = OnceLock::new();

pub fn quality_inspection_fields() -> &'static QualityInspectionFields {
    QUALITY_FIELDS
        .get_or_init(|| {
            load_json("config/fields/quality_inspection_fields.json").unwrap_or_default()
        })
}

#[derive(Deserialize, Clone)]
pub struct MaterialProductionFields {
    pub material_aliases: Vec<String>,
    pub batch_aliases: Vec<String>,
    pub columns: Vec<String>,
}

impl Default for MaterialProductionFields {
    fn default() -> Self {
        Self {
            material_aliases: vec!["物料品号".into(), "物料品名".into()],
            batch_aliases: vec!["批号".into()],
            columns: vec!["物料品号".into(), "生产次数".into()],
        }
    }
}

static MATERIAL_FIELDS: OnceLock<MaterialProductionFields> = OnceLock::new();

pub fn material_production_fields() -> &'static MaterialProductionFields {
    MATERIAL_FIELDS
        .get_or_init(|| {
            load_json("config/fields/material_production_fields.json").unwrap_or_default()
        })
}

// ====================================================================
// 魔数配置
// ====================================================================

#[derive(Deserialize, Clone)]
pub struct PipelineConstants {
    pub raw_line_buffer_size: usize,
    pub data_to_sink_buffer_size: usize,
    pub db_buffer_capacity: usize,
    pub db_flush_threshold: usize,
    pub top_n_count: usize,
    pub top_n_table_name: String,
    pub top_n_columns: Vec<String>,
}

impl Default for PipelineConstants {
    fn default() -> Self {
        Self {
            raw_line_buffer_size: 3000,
            data_to_sink_buffer_size: 3000,
            db_buffer_capacity: 3000,
            db_flush_threshold: 2500,
            top_n_count: 5,
            top_n_table_name: "puaration_top_n".into(),
            top_n_columns: vec![
                "物料品号".into(), "工艺参数向量".into(), "出现次数".into(),
                "不同向量数量".into(), "纯度".into(), "定量IsOK百分比".into(),
                "定性IsOK百分比".into(), "综合得分".into(), "排名".into(),
            ],
        }
    }
}

static PIPELINE_CONSTANTS: OnceLock<PipelineConstants> = OnceLock::new();

pub fn pipeline_constants() -> &'static PipelineConstants {
    PIPELINE_CONSTANTS
        .get_or_init(|| load_json("config/constants/pipeline.json").unwrap_or_default())
}

#[derive(Deserialize, Clone)]
pub struct ScoreConstants {
    pub purity_cap: f64,
    pub pct_divisor: f64,
    pub score_divisor: f64,
    pub default_pass_pct: f64,
    pub empty_value_indicator: String,
    pub qualitative_pass_value: String,
}

impl Default for ScoreConstants {
    fn default() -> Self {
        Self {
            purity_cap: 1.0,
            pct_divisor: 100.0,
            score_divisor: 3.0,
            default_pass_pct: 100.0,
            empty_value_indicator: "None".into(),
            qualitative_pass_value: "OK".into(),
        }
    }
}

static SCORE_CONSTANTS: OnceLock<ScoreConstants> = OnceLock::new();

pub fn score_constants() -> &'static ScoreConstants {
    SCORE_CONSTANTS
        .get_or_init(|| load_json("config/constants/score.json").unwrap_or_default())
}
