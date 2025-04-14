use serde_json::Value;

use super::models::RESPONSE;

pub async fn generate_initialization_response(host: &str) -> Value {
    let response = RESPONSE.replace("{host}", host);
    serde_json::from_str(&response).expect("Failed to parse JSON")
}
