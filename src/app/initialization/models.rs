use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::LazyLock;

pub static INITIALIZATION_RESPONSE: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("response.json"))
        .expect("The initialization response should be valid JSON")
});

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TestsResponse {
    pub result: &'static str,
    pub test_key: String,
    pub tests: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TestRequest {
    pub test_key: String,
}
