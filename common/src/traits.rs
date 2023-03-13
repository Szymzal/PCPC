use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

pub trait PartProperties: Serialize {
    fn to_string_vec(&self) -> anyhow::Result<HashMap<String, String>> {
        let field_values_array = serde_json::to_string(&self.clone())?;
        let mut chars = field_values_array.chars();
        chars.next();
        chars.next_back();
        let field_values_array = format!("[{}]", chars.as_str());
        let field_values_array = field_values_array.replace("\":", "\",");
        let array: Vec<Value> = serde_json::from_str(&field_values_array)?;
        let array: Vec<String> = array.iter().map(|x| x.to_string()).collect();
        let mut map: HashMap<String, String> = HashMap::new();
        for i in 0..array.len() / 2 {
            let key = array.get(i * 2);
            let value = array.get(i * 2 + 1);
            if let (Some(key), Some(value)) = (key, value) {
                let mut key = key.replace("\"", "");
                let mut chars: Vec<char> = key.chars().collect();
                let uppercase_char = chars[0].to_uppercase().nth(0);
                if let Some(uppercase_char) = uppercase_char {
                    chars[0] = uppercase_char;
                    key = chars.into_iter().collect();
                }
                key = key.replace("_", " ");
                let value = value.replace("\"", "");
                map.insert(key.to_string(), value.to_string());
            }
        }

        return Ok(map);
    }
}
