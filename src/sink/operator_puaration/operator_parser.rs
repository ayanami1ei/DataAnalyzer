pub struct OperatorParser;

impl OperatorParser {
    pub fn extract_main_operator(remark: &str) -> Option<String> {
        let key = "主机手";
        let start = remark.find(key)? + key.len();
        let tail = &remark[start..];

        let trimmed = tail.trim_start_matches(|c: char| {
            c.is_whitespace() || c == ':' || c == '：' || c == '-' || c == '—'
        });

        let mut name = String::new();
        for c in trimmed.chars() {
            if Self::is_name_char(c) {
                name.push(c);
                continue;
            }

            if name.is_empty() {
                continue;
            }
            break;
        }

        if name.is_empty() {
            return None;
        }

        Some(name)
    }

    fn is_name_char(c: char) -> bool {
        ('\u{4e00}'..='\u{9fff}').contains(&c) || c.is_ascii_alphabetic() || c == '·'
    }
}
