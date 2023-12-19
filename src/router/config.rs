use axum::response::{IntoResponse, Response};
use axum::Json;
use std::collections::HashMap;

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

impl OpenapiMatchResp {
    pub fn update_formatted_summary(&mut self) {
        let mut summary = self.openapi_log.replace("{ ", "{").replace(" }", "}");
        for (key, value) in self.url_args.iter() {
            summary = summary.replace(&format!("{{{key}}}"), value);
        }

        for body in &self.body_match_list {
            summary = summary.replace(&format!("{{{}}}", body.key), &format!("{}", body.value));
        }
        self.log = summary.replace('"', "");
    }
}

impl IntoResponse for OpenapiMatchResp {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
