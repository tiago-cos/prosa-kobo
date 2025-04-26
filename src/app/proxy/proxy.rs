use axum::{
    body::Bytes,
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
};
use ureq::Agent;

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
    } else if uri.to_string().contains("oeuth") {
        target_uri = format!("{}{}", "https://authorize.kobo.com", &uri.to_string()[6..]);
    } else if uri.to_string().contains("oauth") {
        target_uri = format!("{}{}", "https://oauth.kobo.com", &uri.to_string()[6..]);
    } else {
        target_uri = format!("{}{}", "https://storeapi.kobo.com", uri);
    }

    let agent: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(1)
        .max_redirects_will_error(false)
        .build()
        .into();

    //TODO remove
    println!("{} {}", method, target_uri);

    let mut request_with_body = match method {
        Method::POST => Some(agent.post(&target_uri)),
        Method::PUT => Some(agent.put(&target_uri)),
        Method::PATCH => Some(agent.patch(&target_uri)),
        _ => None,
    };

    let mut request_without_body = match method {
        Method::GET => Some(agent.get(&target_uri)),
        Method::DELETE => Some(agent.delete(&target_uri)),
        _ => None,
    };

    for (key, value) in headers.iter() {
        let key = key.as_str();
        let value = value.to_str().expect("invalid header value");

        if request_with_body.is_some() {
            request_with_body = Some(request_with_body.unwrap().header(key, value));
        }

        //TODO remove
        if target_uri.contains("/rating/") {
            println!("{} {}", key, value);
        }

        if request_without_body.is_some() {
            request_without_body = Some(request_without_body.unwrap().header(key, value));
        }
    }

    /*
    let mut response = if let Some(request) = request_with_body {
        request.send(body.to_vec()).expect("failed to send request")
    } else {
        request_without_body
            .unwrap()
            .call()
            .expect("failed to send request")
    };

    let mut headers = response.headers_mut().clone();

    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::CONTENT_ENCODING);
    headers.remove(header::CONTENT_LENGTH);
    headers.remove(header::CONNECTION);

    let status = response.status();

    let mut body: Vec<u8> = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(50000000)
        .read_to_end(&mut body)
        .expect("failed to read response body");*/

    if method == Method::PATCH {
        return StatusCode::NO_CONTENT;
    }

    StatusCode::NOT_FOUND
    //(status, headers, "").into_response()
}
