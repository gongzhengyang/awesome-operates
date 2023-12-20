use axum::response::{IntoResponse, Response};
use axum::Json;
use std::collections::HashMap;
use once_cell::sync::Lazy;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use regex::Regex;

#[derive(Debug, Deserialize, Serialize, Default, JsonSchema, Clone, PartialEq)]
pub struct OpenapiMatchResp {
    /// the path display on openapi like `/user/:id, /user/list`
    pub openapi_path: String,
    /// request method, like `GET, POST`
    pub method: String,
    pub openapi_log: String,
    pub module: String,
    /// the request body component
    pub component: Option<Value>,
    /// like openapi path, but start with prefix
    pub path_with_prefix: String,
    ///  match "/device/:id/:id2/" with "/device/aaa/bbb/?sasajk" one by one into {"id": "aaa", "id2": "bbb"}
    pub url_args: HashMap<String, String>,
    pub body_match_list: Vec<BodyMatch>,
    /// format original summary by url_args(priority higher) and body value
    pub log: String,
}

#[derive(Debug, Deserialize, Serialize, Default, JsonSchema, Clone, PartialEq)]
pub struct BodyMatch {
    /// body key
    pub key: String,
    /// body key value
    pub value: Value,
    /// the description from openapi component for this key
    pub description: String,
    /// the item key type
    pub value_type: String,
}

/// find
/// ```
/// // "- `1`:   `v1`\n\n- `2`: `v2c`\n\n- `3`: `v3` - 'sa': 'sadas'"
/// ```
/// into `1-v1, 2-v2c, 3-v3`
static ENUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"- {0,3}`(\w+)`: {0,3}`(\w+)`").unwrap());

fn fetch_enum_value_label(description: &str, original_value: String) -> String {
    for (_, [value, label]) in ENUM_RE.captures_iter(description).map(|c| c.extract()) {
        if original_value.trim_matches('"').eq(value) {
            return label.to_owned();
        }
    }
    original_value
}

impl OpenapiMatchResp {
    pub fn update_formatted_summary(&mut self) {
        let mut summary = self.openapi_log.replace("{ ", "{").replace(" }", "}");
        for (key, value) in self.url_args.iter() {
            summary = summary.replace(&format!("{{{key}}}"), value);
        }

        for body in &self.body_match_list {
            let body_value = if let Some(values) = body.value.as_array() {
                let values_formatted = values.into_iter().map(|v| fetch_enum_value_label(&body.description, v.to_string())).collect::<Vec<String>>();
                serde_json::json!(values_formatted).to_string()
            } else {
                fetch_enum_value_label(&body.description, body.value.to_string()).to_owned()
            };
            summary = summary.replace(&format!("{{{}}}", body.key), &format!("{}", body_value));
        }
        self.log = summary.replace('"', "");
    }
}

impl IntoResponse for OpenapiMatchResp {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
