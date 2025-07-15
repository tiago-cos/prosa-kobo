use axum::{
    body::{to_bytes, Body},
    extract::Request,
    middleware::Next,
    response::Response,
};
use log::{error, info};
use tracing_subscriber::{
    fmt::{layer, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use crate::app::error::ErrorResponse;

pub fn init_logging() {
    let fmt_layer = layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(ChronoUtc::rfc_3339())
        .compact();

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let filter = filter.add_directive("html5ever=error".parse().expect("Failed to parse log filter"));

    tracing_subscriber::registry().with(filter).with(fmt_layer).init();
}

pub async fn log_layer(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let mut response = next.run(req).await;

    let status = response.status();

    let colored_code = if status.is_success() || status.is_redirection() {
        format!("\x1B[32m{}\x1B[0m", status.as_u16())
    } else {
        format!("\x1B[31m{}\x1B[0m", status.as_u16())
    };

    if status.is_success() || status.is_redirection() {
        info!("{} {} [{}]", method, path, colored_code);
    } else {
        let headers = response.headers().clone();
        let body = response.into_body();
        let bytes = to_bytes(body, 1000).await.unwrap_or_default();

        let log_message = match serde_json::from_slice::<ErrorResponse>(&bytes) {
            Ok(err) => err.message,
            Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
        };

        if log_message.is_empty() {
            error!("{} {} [{}]", method, path, colored_code);
        } else {
            error!("{} {} [{} - {}]", method, path, colored_code, log_message);
        }

        let mut builder = Response::builder().status(status);
        for (key, value) in headers {
            if let Some(k) = key {
                builder = builder.header(k, value);
            }
        }

        response = builder.body(Body::from(bytes)).unwrap();
    };

    response
}
