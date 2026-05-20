// 主模块：应用程序入口，负责初始化日志、加载配置、构建任务并并发执行数据提取流水线
use std::sync::Arc;

use tracing::Level;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    data::object_io::config_reader::ConfigReader,
    data_creater::WorkRecordCreater,
    excel_reader::ExcelReader,
    extractor_pipeline::ExtractorPipeline,
    router::Router,
    sink::{data_checker::DataChecker, data_sink::DataSink},
};

pub mod checker;
pub mod config;
pub mod data;
pub mod data_creater;
pub mod error;
pub mod excel_reader;
pub mod extractor_pipeline;
pub mod router;
pub mod sink;
pub mod traits;

#[cfg(test)]
pub mod test;

// 任务类型：文件名、数据模板、路由配置路径、是否启用几何校验
type Job = (String, crate::data::data::Data, String, bool);

struct App;

impl App {
    // 设置 tracing 日志系统，将 INFO / WARN / ERROR 级别日志分别写入不同文件，同时将 ERROR 同步输出到 stderr
    fn setup_tracing() -> Vec<tracing_appender::non_blocking::WorkerGuard> {
        let info_a = tracing_appender::rolling::never(".", "logs/info.log");
        let warn_a = tracing_appender::rolling::never(".", "logs/warn.log");
        let error_a = tracing_appender::rolling::never(".", "logs/error.log");

        let (info_w, g1) = tracing_appender::non_blocking(info_a);
        let (warn_w, g2) = tracing_appender::non_blocking(warn_a);
        let (error_w, g3) = tracing_appender::non_blocking(error_a);

        // 按日志级别构建三层过滤器，分别写入对应文件
        let info_l = tracing_subscriber::fmt::layer()
            .with_ansi(false).with_writer(info_w)
            .with_filter(tracing_subscriber::filter::filter_fn(|m| *m.level() == Level::INFO));
        let warn_l = tracing_subscriber::fmt::layer()
            .with_ansi(false).with_writer(warn_w)
            .with_filter(tracing_subscriber::filter::filter_fn(|m| *m.level() == Level::WARN));
        let error_l = tracing_subscriber::fmt::layer()
            .with_ansi(false).with_writer(error_w)
            .with_filter(tracing_subscriber::filter::filter_fn(|m| *m.level() == Level::ERROR));
        // 将 ERROR 级别同时输出到 stderr 以便终端实时查看
        let stderr_l = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);

        tracing_subscriber::registry()
            .with(info_l).with(warn_l).with(error_l).with(stderr_l)
            .init();

        // 返回 WorkerGuard 集合，确保日志缓冲在程序退出前被刷新
        vec![g1, g2, g3]
    }

    // 从 JSON 配置文件中加载 Data 模板对象
    fn load_template(path: &str) -> Result<crate::data::data::Data, crate::error::Error> {
        let p = path.to_string();
        let mut reader = ConfigReader::new(&p);
        reader.load_config()?;
        Ok(reader.get_object())
    }

    // 构建所有待处理任务，包括工序记录（6-8月）和品质检验（06-08月）两类
    fn build_jobs(
        default_tpl: &crate::data::data::Data,
        quality_tpl: &crate::data::data::Data,
    ) -> Vec<Job> {
        let wr_months = ["6", "7", "8"];
        let qa_months = ["06", "07", "08"];
        let mut jobs = Vec::new();
        for m in &wr_months {
            jobs.push((format!("HT_工序纪录明细查询(2024年{}月).xlsx", m), default_tpl.clone(), "config/router_work_record.toml".to_string(), true));
        }
        for m in &qa_months {
            jobs.push((format!("品质检验(2024年{}月).xlsx", m), quality_tpl.clone(), "config/router_quality.toml".to_string(), false));
        }
        jobs
    }

    // 运行单条流水线：加载路由、配置路由器、创建读取器/创建器/检查器，执行流水线
    async fn run_pipeline(
        path: String,
        data_template: crate::data::data::Data,
        router_config_path: String,
        enable_geometry_rule: bool,
    ) -> Result<(), crate::error::Error> {
        let router = Arc::new(Router::new(&router_config_path)?);
        Router::attach_router(&router);
        let reader = ExcelReader::new(&path)?;
        let creater = WorkRecordCreater::new(data_template);
        let checker_config = if enable_geometry_rule {
            "config/sinks/data_checker_work_record.json"
        } else {
            "config/sinks/data_checker_quality.json"
        };
        let checker = DataChecker::new(checker_config);
        checker.set_router(Arc::downgrade(&router));
        let pipeline = ExtractorPipeline::new(reader, creater, Box::new(checker), 0)?;
        pipeline.run().await?;
        router.finish_all();
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    let _guard = App::setup_tracing();
    let default_tpl = App::load_template("config/templates/data_config.json")?;
    let quality_tpl = App::load_template("config/templates/品质检验.json")?;
    let jobs = App::build_jobs(&default_tpl, &quality_tpl);

    let mut tasks = tokio::task::JoinSet::new();
    for (path, tpl, router_cfg, geo) in jobs {
        tasks.spawn(async move {
            App::run_pipeline(path, tpl, router_cfg, geo).await
        });
    }

    // 等待所有任务完成，记录失败的流水线
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::warn!("pipeline failed: {}", e),
            Err(e) => tracing::warn!("pipeline task panicked: {}", e),
        }
    }
    Ok(())
}
