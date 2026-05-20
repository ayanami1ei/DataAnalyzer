// 操作员名称解析器的单元测试
// 测试 OperatorParser::extract_main_operator 从备注信息中提取主机手名称

use crate::sink::operator_puaration::operator_parser::OperatorParser;

#[test]
// 验证：备注中 "主机手XXX" 格式（无冒号分隔）能正确提取操作员姓名
fn extract_main_operator_should_parse_without_colon() {
    let remark = "返工   主机手杜伟   跟班黄刚超";
    let operator_name = OperatorParser::extract_main_operator(remark).expect("operator name");
    assert_eq!(operator_name, "杜伟");
}

#[test]
// 验证：备注中 "主机手：XXX" 格式（含中文冒号）能正确提取操作员姓名
fn extract_main_operator_should_parse_with_chinese_colon() {
    let remark = "普缆  主机手：唐清林 跟班：魏文";
    let operator_name = OperatorParser::extract_main_operator(remark).expect("operator name");
    assert_eq!(operator_name, "唐清林");
}

#[test]
// 验证：备注中缺少 "主机手" 关键字时返回 None
fn extract_main_operator_should_return_none_when_missing_keyword() {
    let remark = "返工 跟班黄刚超";
    assert!(OperatorParser::extract_main_operator(remark).is_none());
}
