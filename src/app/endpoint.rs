use crate::CONFIG;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};

pub struct Host(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Host {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        std::future::ready(
            parts
                .headers
                .get(header::HOST)
                .and_then(|host| host.to_str().ok())
                .or_else(|| {
                    parts
                        .uri
                        .authority()
                        .and_then(|authority| authority.as_str().rsplit('@').next())
                })
                .map(|host| Host(host.to_owned()))
                .ok_or((StatusCode::BAD_REQUEST, "No host found in request")),
        )
    }
}

pub fn device_url(host: &str, lookup_key: &str) -> String {
    format!("{}/{lookup_key}", server_url(host))
}

fn server_url(host: &str) -> String {
    match &CONFIG.server.public {
        Some(public) => format!("{}://{}:{}", public.scheme, public.host, public.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", CONFIG.server.bind.port),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, request::Builder};

    async fn host(request: Builder) -> Result<String, StatusCode> {
        let (mut parts, ()) = request.body(()).expect("Failed to build a request").into_parts();

        Host::from_request_parts(&mut parts, &())
            .await
            .map(|Host(host)| host)
            .map_err(|(status, _)| status)
    }

    #[tokio::test]
    async fn takes_the_host_the_device_addressed() {
        let request = Request::builder().header("host", "192.168.1.20:5001");

        assert_eq!(host(request).await, Ok("192.168.1.20:5001".to_owned()));
    }

    #[tokio::test]
    async fn falls_back_to_the_authority_without_its_credentials() {
        let request = Request::builder().uri("http://user:secret@192.168.1.20:5001/v1/initialization");

        assert_eq!(host(request).await, Ok("192.168.1.20:5001".to_owned()));
    }

    #[tokio::test]
    async fn refuses_a_request_that_names_no_host() {
        let request = Request::builder().uri("/v1/initialization");

        assert_eq!(host(request).await, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn keeps_the_port_the_device_reached_it_on() {
        assert_eq!(
            device_url("middleware.test:8080", "key"),
            "http://middleware.test:8080/key"
        );
    }

    #[test]
    fn names_the_bound_port_when_the_host_carries_none() {
        assert_eq!(
            device_url("middleware.test", "key"),
            format!("http://middleware.test:{}/key", CONFIG.server.bind.port)
        );
    }
}
