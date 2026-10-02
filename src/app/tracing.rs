use crate::app::error::ErrorResponse;
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use log::{debug, error, info};
use tracing_subscriber::{
    EnvFilter,
    fmt::{layer, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Clone, Copy)]
pub struct Unhandled;

#[derive(Clone)]
pub struct LoggedPath(pub String);

pub fn init_logging() {
    let fmt_layer = layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi_sanitization(false)
        .with_timer(ChronoUtc::new("[%Y-%m-%d %H:%M:%S]".to_owned()))
        .compact();

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let filter = filter.add_directive("html5ever=error".parse().expect("Failed to parse log filter"));
    let filter = filter.add_directive("xml5ever=error".parse().expect("Failed to parse log filter"));

    tracing_subscriber::registry().with(filter).with(fmt_layer).init();
}

pub async fn log_layer(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let mut response = next.run(req).await;

    let path = match response.extensions().get::<LoggedPath>() {
        Some(LoggedPath(logged)) => logged.clone(),
        None => path,
    };
    let status = response.status();

    let colored_code = if status.is_success() || status.is_redirection() {
        format!("\x1B[32m{}\x1B[0m", status.as_u16())
    } else {
        format!("\x1B[31m{}\x1B[0m", status.as_u16())
    };

    if path == "/health" || response.extensions().get::<Unhandled>().is_some() {
        debug!("{method} {path} [{colored_code}]");
    } else if status.is_success() || status.is_redirection() {
        info!("{method} {path} [{colored_code}]");
    } else {
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, 1000).await.unwrap_or_default();

        let log_message = match serde_json::from_slice::<ErrorResponse>(&bytes) {
            Ok(err) => err.message,
            Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
        };

        if log_message.is_empty() {
            error!("{method} {path} [{colored_code}]");
        } else {
            error!("{method} {path} [{colored_code} - {log_message}]");
        }

        response = Response::from_parts(parts, Body::from(bytes));
    }

    response
}
