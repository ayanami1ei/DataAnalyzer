// 操作工分离（Operator Puaration）模块的根模块
// 聚合了操作工数据解析、统计、持久化及配置等子模块
pub mod operator_output_metric_columns;
pub mod operator_parser;
pub mod operator_puaration_sink;
pub mod operator_puaration_sink_config;
pub mod operator_puaration_stat;
pub mod sink;

// 对外暴露核心类型：操作工分离Sink与统计记录
pub use operator_puaration_sink::OperatorPuarationSink;
pub use operator_puaration_stat::OperatorPuarationStat;


