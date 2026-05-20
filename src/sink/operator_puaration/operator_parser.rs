// 操作工解析器：从原始备注文本中提取主操作工姓名
pub struct OperatorParser;

impl OperatorParser {
    // 从备注字符串中提取主操作工姓名
    // 查找以"主机手"为关键词的文本段，截取其后的操作工姓名
    // 返回：操作工姓名（Some）或无法提取时返回None
    pub fn extract_main_operator(remark: &str) -> Option<String> {
        // 定位"主机手"关键词，找到其起始位置
        let key = "主机手";
        let start = remark.find(key)? + key.len();
        let tail = &remark[start..];

        // 去除关键词后的空白字符及分隔符号（中文/英文冒号、短横线等）
        let trimmed = tail.trim_start_matches(|c: char| {
            c.is_whitespace() || c == ':' || c == '：' || c == '-' || c == '—'
        });

        // 遍历剩余字符，收集合法姓名部分
        let mut name = String::new();
        for c in trimmed.chars() {
            if Self::is_name_char(c) {
                name.push(c);
                continue;
            }
            // 跳过姓名前的非姓名符号
            if name.is_empty() {
                continue;
            }
            // 遇到第一个非姓名符号时停止收集
            break;
        }

        // 未收集到任何姓名则返回None
        if name.is_empty() {
            return None;
        }

        Some(name)
    }

    // 判断字符是否为合法的姓名组成字符
    // 合法范围：CJK统一表意文字、ASCII字母、间隔号（·）
    fn is_name_char(c: char) -> bool {
        ('\u{4e00}'..='\u{9fff}').contains(&c) || c.is_ascii_alphabetic() || c == '·'
    }
}
