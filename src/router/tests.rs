use axum::body::Body;

use super::*;

fn get_request_matcher() -> (RequestMatcher, Value) {
    let openapi = std::fs::read_to_string("src/test_files/openapi.json").unwrap();
    let openapi = serde_json::from_str::<Value>(&openapi).unwrap();
    (RequestMatcher::from_openapi(&openapi, "").unwrap(), openapi)
}

/// 基础测试
#[tokio::test]
async fn test_basic_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let mut resp = matcher
        .match_request_to_json_response(Method::GET, "/device/", None)
        .await
        .unwrap();
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
            "openapi_path": "/device/",
            "method": "get",
            "url_args": {},
            "path_with_prefix": "/device/",
            "body_match_list": [],
            "summary": "查询设备状态数据 (最多保存历史1000条)",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn test_fetch_url_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let mut resp = matcher
        .match_request_to_json_response(Method::GET, "/device/test-id1/test-id2/", None)
        .await
        .unwrap();
    let url_args = resp.as_object_mut().unwrap().remove("url_args").unwrap();
    assert_eq!(
        url_args,
        serde_json::json!({
            "id": "test-id1",
            "id2": "test-id2"
        })
    );
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
            "openapi_path": "/device/:id/:id2/",
            "method": "get",
            "path_with_prefix": "/device/:id/:id2/",
            "body_match_list": [],
            "summary": "查询设备状态数据 (最多保存历史1000条) id id",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn test_fetch_url_id_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let mut resp = matcher
        .match_request_to_json_response(Method::GET, "/device/22/33/", None)
        .await
        .unwrap();
    let url_args = resp.as_object_mut().unwrap().remove("url_args").unwrap();
    assert_eq!(
        url_args,
        serde_json::json!({
            "id": "22",
            "id2": "33"
        })
    );
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
            "openapi_path": "/device/:id/:id2/",
            "path_with_prefix": "/device/:id/:id2/",
            "method": "get",
            "body_match_list": [],
            "summary": "查询设备状态数据 (最多保存历史1000条) id id",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn test_request_body_openapi() {
    let body = serde_json::json!({
        "community": "public",
        "enabled": true,
        "trap": "1.1.1.1",
        "versions": [1, 2]
    });
    let (mut matcher, openapi) = get_request_matcher();
    let body = Body::from(format!("{body}"));
    let mut resp = matcher
        .match_request_to_json_response(Method::PUT, "/snmpconfig/", Some(body))
        .await
        .unwrap();
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
              "openapi_path": "/snmpconfig/",
               "path_with_prefix": "/snmpconfig/",
              "method": "put",
              "url_args": {},
              "summary": "[snmp] 配置snmp",
              "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("#/components/schemas/SnmpConfig"))),
              "body_match_list": serde_json::json!([
          {
            "description":"community 认证参数",
            "key":"community",
            "type":"string",
            "value":"public"
          },
          {
            "description":"是否开启snmp",
            "key":"enabled",
            "type":"boolean",
            "value":true
          },
          {
            "description":"snmp 远程trap地址",
            "key":"trap",
            "type":"string",
            "value":"1.1.1.1"
          },
          {
            "description":"开启的snmp版本, 列表类型，参数可选项是 1,2,3",
            "key":"versions",
            "type":"array",
            "value": [1, 2]
          }
        ])
          })
    )
}
