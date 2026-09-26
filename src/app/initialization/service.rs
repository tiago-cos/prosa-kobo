use super::models::{INITIALIZATION_RESPONSE, TestsResponse};
use serde_json::{Map, Value};

pub fn generate_initialization_response(host: &str) -> Value {
    let mut response = INITIALIZATION_RESPONSE.clone();
    fill_host(&mut response, host);

    response
}

pub fn generate_tests_response(test_key: &str) -> TestsResponse {
    TestsResponse {
        result: "Success",
        test_key: test_key.to_owned(),
        tests: Map::new(),
    }
}

fn fill_host(value: &mut Value, host: &str) {
    match value {
        Value::String(text) => *text = text.replace("{host}", host),
        Value::Array(items) => items.iter_mut().for_each(|item| fill_host(item, host)),
        Value::Object(fields) => fields.values_mut().for_each(|field| fill_host(field, host)),
        _ => {}
    }
}
