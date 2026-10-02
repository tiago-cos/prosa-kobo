use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub software: &'static str,
    pub version: &'static str,
}

pub const HEALTH_RESPONSE: HealthResponse = HealthResponse {
    status: "ok",
    software: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
