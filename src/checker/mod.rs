// 检查器模块：定义数据校验和几何规则校验的公共接口及实现

pub mod data_check_rule;
pub mod geometry_check_rule;

use data_check_rule::DataCheckRule;

// 根据规则名称字符串创建对应的校验规则实例
pub fn create_rule(name: &str) -> Option<Box<dyn DataCheckRule>> {
    match name {
        "geometry_check" => Some(Box::new(geometry_check_rule::GeometryCheckRule::new())),
        _ => None,
    }
}
