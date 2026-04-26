use crate::errors::HingeError;

pub(super) fn sanitize_component(input: &str) -> String {
    let trimmed = input.trim();
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() { "export".into() } else { out }
}

pub(super) fn parse_ts(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

pub(super) fn parse_json_with_path<T: serde::de::DeserializeOwned>(
    text: &str,
) -> Result<T, HingeError> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
        let path = err.path().to_string();
        if path == "." {
            HingeError::Serde(err.inner().to_string())
        } else {
            HingeError::Serde(format!("{} at {}", err.inner(), path))
        }
    })
}

pub(super) fn parse_json_value_with_path<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, HingeError> {
    parse_json_with_path(&value.to_string())
}

pub(super) fn attachment_from_value(value: &serde_json::Value) -> Option<(String, String)> {
    if !value.is_object() {
        return None;
    }
    let url = value
        .get("url")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("secure_url").and_then(|v| v.as_str()))?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            url.split('/')
                .next_back()
                .unwrap_or("attachment")
                .to_string()
        });
    Some((url.to_string(), name))
}
