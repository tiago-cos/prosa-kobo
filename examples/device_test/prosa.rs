use crate::{
    books::{Cover, Fixture},
    suite::Problem,
};
use prosa_kobo::client::prosa::{Client, ProsaApi};
use reqwest::{Method, RequestBuilder, Response};
use serde::Deserialize;
use serde_json::{Value, json};

const PAGE_SIZE: &str = "100";

pub struct Prosa {
    url: String,
    http: reqwest::Client,
    admin_key: Option<String>,
    pub api: Client,
}

#[derive(Deserialize)]
pub struct Session {
    pub user_id: String,
    pub jwt_token: String,
}

#[derive(Deserialize)]
struct Results {
    #[serde(alias = "shelf_ids")]
    book_ids: Vec<String>,
    total_pages: u64,
    current_page: u64,
}

pub enum Auth<'a> {
    Key(&'a str),
    Bearer(&'a str),
}

impl Prosa {
    pub fn new(url: &str, admin_key: Option<String>) -> Result<Self, Problem> {
        let url = url.trim_end_matches('/').to_owned();
        let (scheme, authority) = url
            .split_once("://")
            .ok_or_else(|| Problem::new(format!("{url} names no scheme")))?;
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => (
                host,
                port.parse()
                    .map_err(|_| Problem::new(format!("{url} names no numeric port")))?,
            ),
            None => (authority, if scheme == "https" { 443 } else { 80 }),
        };

        Ok(Self {
            api: Client::new(scheme, host, port),
            http: reqwest::Client::new(),
            admin_key,
            url,
        })
    }

    pub async fn health(&self) -> Result<(), Problem> {
        self.api
            .health()
            .await
            .map(drop)
            .map_err(|_| Problem::new(format!("Prosa does not answer at {}", self.url)))
    }

    fn request(&self, method: Method, path: &str, auth: &Auth) -> RequestBuilder {
        let request = self.http.request(method, format!("{}{path}", self.url));
        match auth {
            Auth::Key(key) => request.header("api-key", *key),
            Auth::Bearer(jwt) => request.bearer_auth(jwt),
        }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<Session, Problem> {
        let mut request = self.http.post(format!("{}/auth/register", self.url));
        if let Some(admin_key) = &self.admin_key {
            request = request.header("admin-key", admin_key);
        }
        let credentials = json!({ "username": username, "password": password });

        Ok(send(request.json(&credentials)).await?.json().await?)
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Session, Problem> {
        let credentials = json!({ "username": username, "password": password });
        let request = self
            .http
            .post(format!("{}/auth/login", self.url))
            .json(&credentials);

        Ok(send(request).await?.json().await?)
    }

    pub async fn create_key(&self, session: &Session, name: &str) -> Result<String, Problem> {
        let body = json!({ "name": name, "capabilities": ["Read", "Create", "Update", "Delete"] });
        let path = format!("/users/{}/keys", session.user_id);
        let created: Value = send(
            self.request(Method::POST, &path, &Auth::Bearer(&session.jwt_token))
                .json(&body),
        )
        .await?
        .json()
        .await?;

        created["key"]
            .as_str()
            .map(ToOwned::to_owned)
            .ok_or_else(|| Problem::new("Prosa created a key without answering it"))
    }

    pub async fn disable_automatic_metadata(&self, session: &Session) -> Result<(), Problem> {
        let path = format!("/users/{}/preferences", session.user_id);
        let body = json!({ "automatic_metadata": false });
        send(
            self.request(Method::PATCH, &path, &Auth::Bearer(&session.jwt_token))
                .json(&body),
        )
        .await?;

        Ok(())
    }

    pub async fn upload(&self, epub: Vec<u8>, book_id: Option<&str>, key: &str) -> Result<String, Problem> {
        let boundary = "prosa-kobo-device-test";
        let mut body = Vec::new();
        if let Some(book_id) = book_id {
            body.extend(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"book_id\"\r\n\r\n{book_id}\r\n"
                )
                .into_bytes(),
            );
        }
        body.extend(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"epub\"; filename=\"book.epub\"\r\n\
                 Content-Type: application/epub+zip\r\n\r\n"
            )
            .into_bytes(),
        );
        body.extend(epub);
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());

        let request = self
            .request(Method::POST, "/books", &Auth::Key(key))
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body);

        Ok(send(request).await?.text().await?)
    }

    pub async fn book_exists(&self, book_id: &str, key: &str) -> Result<bool, Problem> {
        let response = self
            .request(Method::GET, &format!("/books/{book_id}"), &Auth::Key(key))
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            status => Err(Problem::new(format!(
                "Prosa answered {status} for book {book_id}"
            ))),
        }
    }

    pub async fn books_of(&self, username: &str, key: &str) -> Result<Vec<String>, Problem> {
        self.search("/books", username, key).await
    }

    pub async fn shelves_of(&self, username: &str, key: &str) -> Result<Vec<String>, Problem> {
        self.search("/shelves", username, key).await
    }

    async fn search(&self, path: &str, username: &str, key: &str) -> Result<Vec<String>, Problem> {
        let mut ids = Vec::new();
        let mut page = 1u64;

        loop {
            let request = self.request(Method::GET, path, &Auth::Key(key)).query(&[
                ("username", username),
                ("page", &page.to_string()),
                ("size", PAGE_SIZE),
            ]);
            let result: Results = send(request).await?.json().await?;
            ids.extend(result.book_ids);

            if result.current_page >= result.total_pages {
                return Ok(ids);
            }
            page += 1;
        }
    }

    pub async fn add_metadata(&self, book_id: &str, metadata: &Value, key: &str) -> Result<(), Problem> {
        self.send_metadata(Method::POST, book_id, metadata, key).await
    }

    pub async fn patch_metadata(&self, book_id: &str, metadata: &Value, key: &str) -> Result<(), Problem> {
        self.send_metadata(Method::PATCH, book_id, metadata, key).await
    }

    async fn send_metadata(
        &self,
        method: Method,
        book_id: &str,
        metadata: &Value,
        key: &str,
    ) -> Result<(), Problem> {
        let path = format!("/books/{book_id}/metadata");
        send(self.request(method, &path, &Auth::Key(key)).json(metadata)).await?;

        Ok(())
    }

    pub async fn delete_metadata(&self, book_id: &str, key: &str) -> Result<(), Problem> {
        let path = format!("/books/{book_id}/metadata");
        send(self.request(Method::DELETE, &path, &Auth::Key(key))).await?;

        Ok(())
    }

    pub async fn add_cover(&self, book_id: &str, cover: Cover, key: &str) -> Result<(), Problem> {
        self.send_cover(Method::POST, book_id, cover, key).await
    }

    pub async fn replace_cover(&self, book_id: &str, cover: Cover, key: &str) -> Result<(), Problem> {
        self.send_cover(Method::PUT, book_id, cover, key).await
    }

    async fn send_cover(
        &self,
        method: Method,
        book_id: &str,
        cover: Cover,
        key: &str,
    ) -> Result<(), Problem> {
        let path = format!("/books/{book_id}/cover");
        let request = self
            .request(method, &path, &Auth::Key(key))
            .header("Content-Type", "image/png")
            .body(cover.png());
        send(request).await?;

        Ok(())
    }

    pub async fn delete_cover(&self, book_id: &str, key: &str) -> Result<(), Problem> {
        let path = format!("/books/{book_id}/cover");
        send(self.request(Method::DELETE, &path, &Auth::Key(key))).await?;

        Ok(())
    }

    /// Prosa has no way to swap a book's file, so another device replacing
    /// one looks like this: the book deleted and uploaded again under its id.
    pub async fn stock(&self, fixture: Fixture, book_id: Option<&str>, key: &str) -> Result<String, Problem> {
        let book_id = self.upload(fixture.book().to_epub(), book_id, key).await?;

        if let Some(metadata) = fixture.metadata() {
            self.add_metadata(&book_id, &metadata, key).await?;
        }
        if let Some(cover) = fixture.cover() {
            self.add_cover(&book_id, cover, key).await?;
        }

        Ok(book_id)
    }
}

async fn send(request: RequestBuilder) -> Result<Response, Problem> {
    let response = request.send().await?;
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let url = response.url().path().to_owned();
    let body = response.text().await.unwrap_or_default();
    Err(Problem::new(format!("{url} answered {status}: {body}")))
}
