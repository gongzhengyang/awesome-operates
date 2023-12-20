use axum::body::Body;
use axum::response::{IntoResponse, Response};
use axum::Json;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::router::GLOBAL_PREFIX_OPENAPI;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    /// openapi prefix
    pub prefix: String,
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
                let values_formatted = values
                    .into_iter()
                    .map(|v| fetch_enum_value_label(&body.description, v.to_string()))
                    .collect::<Vec<String>>();
                serde_json::json!(values_formatted).to_string()
            } else {
                fetch_enum_value_label(&body.description, body.value.to_string()).to_owned()
            };
            summary = summary.replace(&format!("{{{}}}", body.key), &format!("{}", body_value));
        }
        self.log = summary.replace('"', "");
        tracing::debug!(
            "update log from {} to {} with url_args: {:?} body_match_list: {:?}",
            self.openapi_log,
            self.log,
            self.url_args,
            self.body_match_list
        );
    }

    /// match body properties field by field with component with body
    pub async fn match_body_args(&mut self, body: Body) {
        if self.component.is_none() {
            return;
        }
        let bytes = http_body_util::BodyExt::collect(body)
            .await
            .unwrap()
            .to_bytes();
        tracing::debug!("handle openapi request receive body len {}", bytes.len());
        let json_body = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            tracing::info!("body transfer is not json for {bytes:?}");
            serde_json::json!({})
        });

        let mut resp = vec![];
        if let Some(properties) = self.component.clone().unwrap()["properties"].as_object() {
            for (key, value) in properties.iter() {
                let body_value = &json_body[key];
                if !body_value.is_null() {
                    resp.push(BodyMatch {
                        key: key.clone(),
                        value: body_value.clone(),
                        description: self.fetch_value_with_ref_check(value, "description").await,
                        value_type: self.fetch_value_with_ref_check(value, "type").await,
                    });
                }
            }
        }

        self.body_match_list = resp;
    }

    /// if the component is only one key eq $ref, return child key value
    async fn fetch_value_with_ref_check(&self, value: &Value, key: &str) -> String {
        let resp = if value
            .as_object()
            .is_some_and(|x| x.keys().collect::<Vec<_>>().len().eq(&1))
        {
            self.fetch_from_openapi_ref(value, key).await
        } else {
            Self::pointer_for_string(value, key)
        };
        println!("fetch from {value} with key {key} to {resp:?}");
        resp.unwrap_or_else(|| "".to_owned())
    }

    fn pointer_for_string(value: &Value, key: &str) -> Option<String> {
        let key = format!("/{}", key.trim_start_matches('/'));
        Some(value.pointer(&key)?.as_str()?.to_owned())
    }

    async fn fetch_from_openapi_ref(&self, component: &Value, key: &str) -> Option<String> {
        let path = component["$ref"].as_str()?.trim_start_matches('#');
        Self::pointer_for_string(
            GLOBAL_PREFIX_OPENAPI
                .read()
                .await
                .get(&self.prefix)?
                .pointer(path)?,
            key,
        )
    }
}

impl IntoResponse for OpenapiMatchResp {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
