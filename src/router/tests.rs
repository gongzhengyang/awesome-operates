use axum::body::Body;

use super::*;

fn get_request_matcher() -> (RequestMatcher, Value) {
    let openapi = std::fs::read_to_string("src/test_files/openapi.json").unwrap();
    let openapi = serde_json::from_str::<Value>(&openapi).unwrap();
    (RequestMatcher::from_openapi(&openapi, "").unwrap(), openapi)
}

#[tokio::test]
async fn router_not_exists() {
    let (mut matcher, _) = get_request_matcher();
    let resp = matcher
        .match_request_to_response(Method::GET, "/not-exists/", None)
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn router_basic_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let resp = matcher
        .match_request_to_response(Method::GET, "/device/", None)
        .await
        .unwrap();

    assert_eq!(
        serde_json::json!(resp),
        serde_json::json!({
            "openapi_path": "/device/",
            "method": "get",
            "url_args": {},
            "path_with_prefix": "/device/",
            "body_match_list": [],
            "module": "设备状态查询",
            "log": "查询设备状态数据 (最多保存历史1000条)",
            "openapi_log": "查询设备状态数据 (最多保存历史1000条)",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn router_fetch_url_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let resp = matcher
        .match_request_to_response(Method::GET, "/device/test-id1/test-id2/", None)
        .await
        .unwrap();
    assert_eq!(
        serde_json::json!(resp),
        serde_json::json!({
            "openapi_path": "/device/:id/:id2/",
            "method": "get",
            "url_args": {
                "id": "test-id1",
                "id2": "test-id2"
            },
            "path_with_prefix": "/device/:id/:id2/",
            "body_match_list": [],
            "log": "",
            "module": "",
            "openapi_log": "",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn router_fetch_url_id_openapi() {
    let (mut matcher, openapi) = get_request_matcher();
    let resp = matcher
        .match_request_to_response(Method::GET, "/device/22/33/", None)
        .await
        .unwrap();
    assert_eq!(
        serde_json::json!(resp),
        serde_json::json!({
            "openapi_path": "/device/:id/:id2/",
            "path_with_prefix": "/device/:id/:id2/",
            "method": "get",
            "url_args": {
                "id": "22",
                "id2": "33"
            },
              "module": "",
            "body_match_list": [],
            "log": "",
             "openapi_log": "",
            "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("")))
        })
    );
}

#[tokio::test]
async fn router_request_body_openapi() {
    let body = serde_json::json!({
        "community": "public",
        "enabled": true,
        "trap": "1.1.1.1",
        "versions": [1, 2]
    });
    let (mut matcher, openapi) = get_request_matcher();
    let body = Body::from(format!("{body}"));
    let resp = matcher
        .match_request_to_response(Method::PUT, "/snmpconfig/", Some(body))
        .await
        .unwrap();
    assert_eq!(
        serde_json::json!(resp),
        serde_json::json!({
              "openapi_path": "/snmpconfig/",
              "path_with_prefix": "/snmpconfig/",
              "method": "put",
              "url_args": {},
            "module": "snmp",
              "log": "配置snmp的认证参数community为public, snmp状态: true, 版本信息是: [1,2]",
              "openapi_log": "配置snmp的认证参数community为{community}, snmp状态: {enabled}, 版本信息是: {versions}",
              "component": RequestMatcher::api_component(&openapi, Some(&serde_json::json!("#/components/schemas/SnmpConfig"))),
              "body_match_list": serde_json::json!([
          {
            "description":"community 认证参数",
            "key":"community",
            "value_type":"string",
            "value":"public"
          },
          {
            "description":"是否开启snmp",
            "key":"enabled",
            "value_type":"boolean",
            "value":true
          },
          {
            "description":"snmp 远程trap地址",
            "key":"trap",
            "value_type":"string",
            "value":"1.1.1.1"
          },
          {
            "description":"开启的snmp版本, 列表类型，参数可选项是 1,2,3",
            "key":"versions",
            "value_type":"array",
            "value": [1, 2]
          }
        ])
          })
    )
}
