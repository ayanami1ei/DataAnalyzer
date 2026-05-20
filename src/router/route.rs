// 路由定义：描述一条数据分发规则，包含匹配条件和目标接收端名称

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

// 路由规则：当数据的流状态集合与 when 匹配时，将数据路由至 to 指定的接收端
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    // 触发路由的流状态集合（BTreeSet<String>），数据必须包含所有这些状态
    pub(super) when: BTreeSet<String>,
    // 目标数据接收端的注册名称
    pub(super) to: String,
}

impl Route {
    // 创建一条新的路由规则
    pub fn new(when: BTreeSet<String>, to: impl Into<String>) -> Self {
        Self {
            when,
            to: to.into(),
        }
    }
}
