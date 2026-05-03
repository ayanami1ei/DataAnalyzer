pub mod stats_flow_config;
pub mod stats_flow_sink;

pub use stats_flow_sink::StatsFlowSink;

pub(crate) const STATS_FLOW_CONFIG_PATH: &str = "config/stats/sink_flow.json";
