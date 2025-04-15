use axum::{
    body::Bytes,
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::IntoResponse,
};
use ureq::request;
use std::io::Read;

pub async fn proxy_handler(
    method: Method,
    uri: Uri,
    mut headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    headers.remove(header::HOST);

    let target_uri: String;
    if uri.to_string().contains("book-images") {
        target_uri = format!("{}{}", "https://cdn.kobo.com", uri);
    } else {
        target_uri = format!("{}{}", "https://storeapi.kobo.com", uri);
    }

    let mut request = request(method.as_str(), &target_uri);
    for (key, value) in headers.iter() {
        request = request.set(key.as_str(), value.to_str().expect("invalid header value"));
    }

    let response = if !body.is_empty() {
        request.send_bytes(&body).expect("failed to send request")
    } else {
        request.call().expect("failed to send request")
    };

    let mut headers = HeaderMap::new();
    for header in response.headers_names() {
        let value = response.header(&header).expect("header not found");
        headers.insert(
            HeaderName::try_from(&header).expect("invalid header name"),
            HeaderValue::from_str(value).expect("invalid header value"),
        );
    }

    headers.remove(header::TRANSFER_ENCODING);

    let status_code = StatusCode::from_u16(response.status()).expect("invalid status code");

    let mut body: Vec<u8> = Vec::new();
    response
        .into_reader()
        .take(50000000)
        .read_to_end(&mut body)
        .expect("failed to read response body");

    (status_code, headers, body)
}