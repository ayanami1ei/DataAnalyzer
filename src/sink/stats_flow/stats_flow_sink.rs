use crate::{
    data::data::Data,
    error::Error,
    sink::{
        data_sink::DataSink,
        end_sink::EndSinkType,
        material_production::MaterialProductionSink,
        operator_puaration::OperatorPuarationSink,
        puaration::PuarationSink,
        stats_flow::{STATS_FLOW_CONFIG_PATH, stats_flow_config::StatsFlowConfig},
    },
};

pub struct StatsFlowSink<SinkType: DataSink> {
    next_sink: SinkType,
    sink_order: Vec<String>,
    puaration_sink: PuarationSink<EndSinkType>,
    operator_puaration_sink: OperatorPuarationSink<EndSinkType>,
    material_production_sink: MaterialProductionSink<EndSinkType>,
}

impl<SinkType: DataSink> StatsFlowSink<SinkType> {
    fn load_config() -> StatsFlowConfig {
        let Ok(text) = std::fs::read_to_string(STATS_FLOW_CONFIG_PATH) else {
            return StatsFlowConfig::default();
        };

        serde_json::from_str::<StatsFlowConfig>(&text).unwrap_or_default()
    }

    pub fn new(next_sink: SinkType) -> Self {
        let cfg = Self::load_config();

        Self {
            next_sink,
            sink_order: cfg.sink_order,
            puaration_sink: PuarationSink::new(EndSinkType {}),
            operator_puaration_sink: OperatorPuarationSink::new(EndSinkType {}),
            material_production_sink: MaterialProductionSink::new(EndSinkType {}),
        }
    }

    fn sink_one_step(&mut self, step: &str, data: &mut Data) -> Result<(), Error> {
        if step == "puaration" {
            return self.puaration_sink.sink(data);
        }
        if step == "operator_puaration" {
            return self.operator_puaration_sink.sink(data);
        }
        if step == "material_production" {
            return self.material_production_sink.sink(data);
        }

        crate::log::append_log_line(&format!(
            "stats flow warning: unknown sink step [{}], skipped",
            step
        ));
        Ok(())
    }
}

impl<SinkType: DataSink> DataSink for StatsFlowSink<SinkType> {
    type NextType = SinkType;

    fn sink(&mut self, data: &mut Data) -> Result<(), Error> {
        for step in self.sink_order.clone() {
            self.sink_one_step(&step, data)?;
        }

        self.next_sink.sink(data)
    }

    fn get_next_sink(&self) -> Result<Option<Self::NextType>, Error> {
        Ok(None)
    }
}
