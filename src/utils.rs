#[macro_export]
macro_rules! parse_json {
    ($text:expr) => {{
        let text = &$text;
        serde_json::from_str::<serde_json::Value>(text).expect(text)
    }};
}

#[macro_export]
macro_rules! json_at {
    ($json:expr, $key:expr) => {{
        let json = &$json;
        let key = &$key;
        let value = &json[key];
        if !value.is_null() {
            Ok(value)
        } else {
            Err(format!("missing key: {}, in {}", key, json))
        }
    }};
    ($json:expr, $key:expr, $($keys:expr),+) => {{
        json_at!($json, $key).and_then(|x| json_at!(x, $($keys),+))
    }};
}

#[macro_export]
macro_rules! json_value_as_i64 {
    ($json:expr) => {{
        let value = &$json;
        match value {
            serde_json::Value::Number(val) => Ok(val.as_i64().unwrap()),
            _ => Err(format!("cannot convert to int: {}", value)),
        }
    }};
}

#[macro_export]
macro_rules! json_value_as_vec {
    ($json:expr) => {{
        let value = &$json;
        match value {
            serde_json::Value::Array(val) => Ok(val),
            _ => Err(format!("cannot convert to array: {}", value)),
        }
    }};
}

#[macro_export]
macro_rules! json_value_as_str {
    ($json:expr) => {{
        let value = &$json;
        match value {
            serde_json::Value::String(val) => Ok(val.as_str()),
            _ => Err(format!("cannot convert to str: {}", value)),
        }
    }};
}
