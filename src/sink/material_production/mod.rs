// 物料生产统计模块：统计各物料的生产次数并持久化到数据库
pub mod material_production_sink;
pub mod material_production_stat;

pub use material_production_sink::MaterialProductionSink;
pub use material_production_stat::MaterialProductionStat;
