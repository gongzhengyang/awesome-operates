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
    println!("get /device/{resp:?}");
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
            "openapi_path": "/device/",
            "method": "get",
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
    println!("get /device/test-id1/test-id2/ {resp:?}");
    resp.as_object_mut().unwrap().remove("request");
    assert_eq!(
        resp,
        serde_json::json!({
            "openapi_path": "/device/:id/:id2/",
            "method": "get",
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
        "version": [1, 2]
    });
    let (mut matcher, _openapi) = get_request_matcher();
    let resp = matcher
        .match_request_to_json_response(Method::PUT, "/snmpconfig/", Some(body))
        .await
        .unwrap();
    println!("put /snmpconfig/ {resp:?}");
}
