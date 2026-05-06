use crate::{
    checker::geometry_check_rule::GeometryCheckRule,
    data::object_io::config_reader::ConfigReader,
    data_creater::WorkRecordCreater,
    excel_reader::ExcelReader,
    extractor_pipeline::ExtractorPipeline,
    sink::{
        data_checker::DataChecker, database::sqlite_database_sink::SqliteDatabaseSink,
        end_sink::EndSinkType, stats_flow::StatsFlowSink,
    },
};

pub mod checker;
pub mod data;
pub mod data_creater;
pub mod error;
pub mod excel_reader;
pub mod extractor_pipeline;
pub mod log;
pub mod progress_table;
pub mod sink;
pub mod traits;
pub mod router;

/*
fn main() {
    // 构造最小数据集，验证 Data 的增删改查与序列化写文件链路。
    let mut data = Data::new("aaa".to_string());
    data.add_key("key1".to_string());
    data.add_key("key2".to_string());

    let writer = ConfigWriter::new(data.clone());
    writer.save_config().unwrap();

    let mut reader= crate::data::object_io::config_reader::ConfigReader::new(&data.name);
    reader.load_config().unwrap();
    let loaded_data=reader.get_object();
    println!("loaded data: {:?}", loaded_data);
}

*/
#[cfg(test)]
pub mod test;

fn main() -> Result<(), error::Error> {
    let default_data_config_path = "config/templates/data_config.json".to_string();
    let quality_data_config_path = "config/templates/品质检验.json".to_string();
    let database_config_path = "config/database/database_config.json".to_string();
    let quality_database_config_path = "config/database/quality_database_config.json".to_string();
    let mut default_config_reader = ConfigReader::new(&default_data_config_path);
    default_config_reader.load_config()?;
    let default_data_template = default_config_reader.get_object();

    let mut quality_config_reader = ConfigReader::new(&quality_data_config_path);
    quality_config_reader.load_config()?;
    let quality_data_template = quality_config_reader.get_object();

    let jobs = vec![
        (
            "HT_工序纪录明细查询(2024年6月).xlsx".to_string(),
            default_data_template.clone(),
            database_config_path.clone(),
            true,
        ),
        (
            "HT_工序纪录明细查询(2024年7月).xlsx".to_string(),
            default_data_template.clone(),
            database_config_path.clone(),
            true,
        ),
        (
            "HT_工序纪录明细查询(2024年8月).xlsx".to_string(),
            default_data_template.clone(),
            database_config_path.clone(),
            true,
        ),
        (
            "品质检验(2024年06月).xlsx".to_string(),
            quality_data_template.clone(),
            quality_database_config_path.clone(),
            false,
        ),
        (
            "品质检验(2024年07月).xlsx".to_string(),
            quality_data_template.clone(),
            quality_database_config_path.clone(),
            false,
        ),
        (
            "品质检验(2024年08月).xlsx".to_string(),
            quality_data_template.clone(),
            quality_database_config_path.clone(),
            false,
        ),
    ];
    let indexing_row = 0usize;

    // 各文件列顺序可能不同，按文件独立运行 pipeline，避免跨文件复用单一表头映射。
    for (path, data_template, db_config_path, enable_geometry_rule) in jobs {
        let reader = ExcelReader::new(&path)?;
        let creater = WorkRecordCreater::new(data_template);
        let end_sink = Box::new(EndSinkType {});
        let stats_flow_sink = StatsFlowSink::new(end_sink);
        let sqlite_sink = SqliteDatabaseSink::new(Box::new(stats_flow_sink), &db_config_path)?;
        let rules = if enable_geometry_rule {
            vec![Box::new(GeometryCheckRule::new())
                as Box<dyn crate::checker::data_check_rule::DataCheckRule>]
        } else {
            Vec::new()
        };
        let checker = DataChecker::new(Box::new(sqlite_sink), rules);
        let pipeline = ExtractorPipeline::new(reader, creater, Box::new(checker), indexing_row)?;
        pipeline.run()?;
    }

    Ok(())
}
