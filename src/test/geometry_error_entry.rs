pub struct GeometryErrorEntry {
    pub batch_no: String,
    pub material_name: String,
    pub error_detail: String,
}

impl GeometryErrorEntry {
    pub fn parse_from_log_line(line: &str) -> Option<Self> {
        if !line.contains("data checker warning: key[") {
            return None;
        }
        let key_start = line.find("key[")? + 4;
        let detail_sep = line.find("], data error:")?;
        let key_part = &line[key_start..detail_sep];
        let error_detail = line[(detail_sep + "], data error:".len())..].trim();

        let batch_no = Self::extract_key_value(key_part, "批号")?;
        let material_name = Self::extract_key_value(key_part, "物料品名")?;

        Some(Self {
            batch_no,
            material_name,
            error_detail: error_detail.to_string(),
        })
    }

    fn extract_key_value(key_part: &str, key_name: &str) -> Option<String> {
        let marker = format!("{}:", key_name);
        let start = key_part.find(&marker)? + marker.len();
        let remain = key_part[start..].trim_start();
        let end = remain.find(',').unwrap_or(remain.len());
        Some(remain[..end].trim().to_string())
    }
}
