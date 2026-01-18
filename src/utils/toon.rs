//! Token-Oriented Object Notation (TOON) Formatter
//! 
//! Provides a compact, non-redundant serialization format for LLMs
//! to reduce token consumption compared to standard JSON.

use serde_json::Value;

pub struct ToonFormatter;

impl ToonFormatter {
    /// Convert JSON to compact TOON notation
    pub fn format(value: &Value) -> String {
        Self::format_recursive(value, 0)
    }

    fn format_recursive(value: &Value, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => {
                // If it contains newlines, use a block-style quote if it helps, 
                // otherwise just return the string.
                if s.contains('\n') {
                    format!("|\n{}", s.lines().map(|l| format!("{}  {}", indent_str, l)).collect::<Vec<_>>().join("\n"))
                } else {
                    format!("\"{}\"", s)
                }
            },
            Value::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                
                // SOTA: Tabular Array Optimization
                // If all elements are objects with the same keys, use TOON tabular format
                if let Some(first) = arr.first() {
                    if first.is_object() {
                        let keys = first.as_object().unwrap().keys().collect::<Vec<_>>();
                        let all_match = arr.iter().all(|v| {
                            v.is_object() && v.as_object().unwrap().keys().collect::<Vec<_>>() == keys
                        });

                        if all_match {
                            let mut out = format!("[#{} {}:]\n", arr.len(), keys.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(", "));
                            for item in arr {
                                let values = keys.iter().map(|k| {
                                    let val = &item[*k];
                                    if val.is_string() { val.as_str().unwrap().to_string() }
                                    else { val.to_string() }
                                }).collect::<Vec<_>>().join(" | ");
                                out.push_str(&format!("{}  - {}\n", indent_str, values));
                            }
                            return out.trim_end().to_string();
                        }
                    }
                }

                // Standard array
                let mut out = String::from("[
");
                for item in arr {
                    out.push_str(&format!("{}  - {}\n", indent_str, Self::format_recursive(item, indent + 1)));
                }
                out.push_str(&format!("{}]", indent_str));
                out
            },
            Value::Object(map) => {
                if map.is_empty() {
                    return "{}".to_string();
                }
                let mut out = String::new();
                for (key, val) in map {
                    out.push_str(&format!("{}{}: {}\n", indent_str, key, Self::format_recursive(val, indent + 1)));
                }
                out.trim_end().to_string()
            }
        }
    }
}
