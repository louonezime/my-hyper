fn get_string_quotes(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.split('"').collect();

    if parts.len() >= 3 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

pub fn inspect_fetch (line: &&str) -> Option<String> {
    if let Some(callout) = line.rfind("fetch(") {
        let (_, param_val) = line.split_at(callout);
        if let Some(link) = param_val.rfind("\"/") {
            let (_, full_val) = param_val.split_at(link);
            return get_string_quotes(full_val);
        }
    }
    return None;
}
