use super::models::{RESPONSE, TESTS_RESPONSE};
use serde_json::Value;

pub fn generate_initialization_response(host: &str) -> Value {
    let response = RESPONSE.replace("{host}", host);

    serde_json::from_str(&response).expect("Failed to parse JSON")
}

pub fn generate_tests_response(test_key: &str) -> Value {
    let response = TESTS_RESPONSE.replace("{test_key}", test_key);

    serde_json::from_str(&response).expect("Failed to parse JSON")
}
