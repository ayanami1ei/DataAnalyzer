// 数据检查器：对流入的数据执行校验规则检查，根据检查结果标记数据有效/无效状态，并路由到下游
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use sink_macro::DataSink;
use tracing::{info, warn};

use crate::{
    checker, checker::data_check_rule::DataCheckRule, error::Error, router::Router,
    sink::data_sink::DataSink,
};
use std::sync::Weak;

#[derive(Deserialize)]
struct DataCheckerConfig {
    rules: Vec<String>,
}

// 数据检查器：持有一组校验规则和一个路由器，依次对数据执行校验并标记状态
#[derive(DataSink)]
pub struct DataChecker {
    // 校验规则列表，按顺序依次执行
    rules: Vec<Box<dyn DataCheckRule>>,
    router: Mutex<Option<Weak<Router>>>,
}

impl DataChecker {
    pub fn new(config_path: &str) -> Self {
        let text = std::fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("DataChecker: cannot read config: {}", config_path));
        let cfg: DataCheckerConfig = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("DataChecker: invalid config: {}", config_path));
        let rules: Vec<Box<dyn DataCheckRule>> = cfg
            .rules
            .iter()
            .filter_map(|name| checker::create_rule(name))
            .collect();
        Self {
            rules,
            router: Mutex::new(None),
        }
    }

    // 获取数据的对象键标识字符串，用于日志输出
    fn key_identity(data: &crate::data::data::Data) -> String {
        data.object_key_str().unwrap_or_default()
    }

    // 对单条数据执行所有校验规则，并记录校验失败信息
    fn check_data(data_guard: &mut crate::data::data::Data, rules: &[Box<dyn DataCheckRule>]) {
        // 重置数据的校验状态
        data_guard.reset_validation_state();
        // 检查数据是否就绪，未就绪的数据直接标记为无效
        if !data_guard.is_ready() {
            let err = Error::DataNotReady(data_guard.get_key_str().unwrap_or_default()).to_string();
            warn!("data checker warning: key[{}], {}", Self::key_identity(data_guard), err);
            data_guard.mark_invalid(err);
        }
        // 依次执行每条校验规则
        for rule in rules {
            if let Err(e) = rule.check(data_guard) {
                let msg = e.to_string();
                warn!("data checker warning: key[{}], {}", Self::key_identity(data_guard), msg);
                data_guard.mark_invalid(msg);
            }
        }
    }

    // 完成数据校验后的状态标记：添加"checked"状态，并根据有效性添加"valid"或"invalid"状态
    fn finalize_state(data_guard: &mut crate::data::data::Data) {
        data_guard.add_flow_state("checked".to_string());
        if data_guard.is_valid() {
            data_guard.add_flow_state("valid".to_string());
        } else {
            data_guard.add_flow_state("invalid".to_string());
            data_guard.remove_flow_state("valid");
        }
    }
}



#[async_trait]
impl DataSink for DataChecker {
    fn new(config_path: &str) -> Self {
        Self::new(config_path)
    }

    fn set_router(&self, router: Weak<Router>) {
        *self.router.lock().unwrap() = Some(router);
    }

    async fn sink(&self, data: Arc<Mutex<crate::data::data::Data>>) -> Result<(), Error> {
        {
            let mut data_guard = data
                .lock()
                .map_err(|_| Error::PipelineError("data mutex poisoned".into()))?;
            Self::check_data(&mut data_guard, &self.rules);
            Self::finalize_state(&mut data_guard);
            info!(
                "data_checker: done, flow_state={:?}, is_valid={}, is_ready={}",
                data_guard.flow_state_set(),
                data_guard.is_valid(),
                data_guard.is_ready()
            );
        }
        let router = self.router.lock().unwrap().as_ref().and_then(|w| w.upgrade());
        if let Some(router) = router {
            router.route_data(Arc::clone(&data)).await?;
        }
        Ok(())
    }
}
