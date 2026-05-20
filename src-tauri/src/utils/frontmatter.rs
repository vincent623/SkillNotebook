pub fn extract(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let mut lines = trimmed.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut block = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            return Some(block.join("\n"));
        }
        block.push(line.to_string());
    }
    None
}

pub fn get_value(block: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = block.lines().collect();
    let prefix = format!("{}:", key);
    let mut index = 0;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let value = rest.trim();
            if value == "|" || value == ">" {
                index += 1;
                let mut block_lines = Vec::new();
                while index < lines.len() {
                    let current = lines[index];
                    if current.starts_with(' ') || current.starts_with('\t') {
                        block_lines.push(current.trim().to_string());
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(block_lines.join(" "));
            }
            let cleaned = value.trim_matches('"').trim_matches('\'').to_string();
            return if cleaned.is_empty() { None } else { Some(cleaned) };
        }
        index += 1;
    }
    None
}
