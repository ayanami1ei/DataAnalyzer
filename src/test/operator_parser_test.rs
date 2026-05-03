use crate::sink::operator_puaration::operator_parser::OperatorParser;

#[test]
fn extract_main_operator_should_parse_without_colon() {
    let remark = "返工   主机手杜伟   跟班黄刚超";
    let operator_name = OperatorParser::extract_main_operator(remark).expect("operator name");
    assert_eq!(operator_name, "杜伟");
}

#[test]
fn extract_main_operator_should_parse_with_chinese_colon() {
    let remark = "普缆  主机手：唐清林 跟班：魏文";
    let operator_name = OperatorParser::extract_main_operator(remark).expect("operator name");
    assert_eq!(operator_name, "唐清林");
}

#[test]
fn extract_main_operator_should_return_none_when_missing_keyword() {
    let remark = "返工 跟班黄刚超";
    assert!(OperatorParser::extract_main_operator(remark).is_none());
}
