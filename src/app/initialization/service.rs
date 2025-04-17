use super::models::{RESPONSE, TESTS_RESPONSE};
use serde_json::Value;
use urlencoding::encode;

pub async fn generate_initialization_response(host: &str, device_id: &str) -> Value {
    let device_id = encode(device_id).to_string();
    let response = RESPONSE.replace("{host}", host);
    let response = response.replace("{device_id}", &device_id);
    serde_json::from_str(&response).expect("Failed to parse JSON")
}

pub async fn generate_tests_response(test_key: &str) -> String {
    TESTS_RESPONSE.replace("{test_key}", test_key)
}
