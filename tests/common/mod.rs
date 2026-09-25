#![allow(dead_code)]

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use prosa_kobo::{
    app::{
        KEPUBS,
        authentication::{
            models::{PROSA_ISSUER, ProsaJWTClaims, ProsaRole},
            service::load_prosa_keys,
        },
        devices, router,
    },
    client::{
        ProsaReadingStatus,
        identity::{ProsaAuthType, ProsaIdentity},
        mock::MockProsaClient,
        prosa::ProsaApi,
        set_prosa_client,
        state::{ProsaState, ProsaStatistics},
    },
    database,
};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use tower::ServiceExt;

pub const API_KEY: &str = "anapikey";
pub const USER: &str = "user";
pub const USERNAME: &str = "reader";
pub const DEVICE_HARDWARE_ID: &str = "N123456789012";
pub const HOST: &str = "middleware.test:5001";

/// A middleware wired to an in-memory Prosa and a database of its own, so a
/// test can drive the real router without either running anywhere.
pub struct Harness {
    pub client: Arc<MockProsaClient>,
    _in_use: MutexGuard<'static, ()>,
}

impl Harness {
    pub async fn new() -> Self {
        static SHARED: OnceCell<Arc<MockProsaClient>> = OnceCell::const_new();
        static IN_USE: Mutex<()> = Mutex::const_new(());

        let in_use = IN_USE.lock().await;

        let client = SHARED
            .get_or_init(|| async {
                let file = format!(
                    "{}/prosa-kobo-{}.db",
                    env!("CARGO_TARGET_TMPDIR"),
                    std::process::id()
                );
                let pool = database::init(&file)
                    .await
                    .expect("Failed to create the test database");
                database::set_pool(pool).expect("Failed to install the test database");

                let client = Arc::new(MockProsaClient::new());
                set_prosa_client(Arc::clone(&client) as Arc<dyn ProsaApi>)
                    .expect("Failed to install the mock Prosa client");

                client
            })
            .await;

        reset_database().await;
        client.reset();
        KEPUBS.clear();

        // A suite that signs its own tokens seeds the matching key set instead.
        client.seed_jwks(signing_keys());
        load_prosa_keys(client.as_ref()).await;

        Self {
            client: Arc::clone(client),
            _in_use: in_use,
        }
    }

    pub fn add_book(&self, book_id: &str) {
        self.client
            .seed_file(book_id, fixture("The_Great_Gatsby.epub"))
            .seed_state(
                book_id,
                ProsaState {
                    location: None,
                    statistics: ProsaStatistics {
                        rating: None,
                        reading_status: ProsaReadingStatus::Unread,
                    },
                },
            );
    }

    pub async fn fail_writes_to(&self, table: &str) {
        for operation in ["INSERT", "UPDATE", "DELETE"] {
            sqlx::query(&format!(
                "CREATE TRIGGER fail_{operation}_{table} BEFORE {operation} ON {table} \
                 BEGIN SELECT RAISE(ABORT, 'writes to {table} fail in this test'); END"
            ))
            .execute(database::pool())
            .await
            .expect("Failed to make writes fail");
        }
    }

    pub async fn unlink(&self, device_id: &str) {
        devices::service::unlink_device(device_id)
            .await
            .expect("Failed to unlink the device");
    }

    pub async fn link(&self, name: &str, api_key: &str) -> Device {
        self.recognize(api_key);

        let (device_id, lookup_key) =
            devices::service::link_device(self.client.as_ref(), USER, name, api_key)
                .await
                .expect("Failed to link the device");

        Device {
            device_id,
            lookup_key,
        }
    }

    pub fn recognize(&self, api_key: &str) {
        self.client.seed_identity(
            api_key,
            ProsaIdentity {
                auth_type: ProsaAuthType::ApiKey,
                user_id: USER.to_owned(),
                username: USERNAME.to_owned(),
                is_admin: false,
                capabilities: vec!["Read".to_owned(), "Create".to_owned(), "Update".to_owned()],
                key_id: Some("key".to_owned()),
            },
        );
    }

    pub async fn linked(&self) -> Device {
        self.link("Kobo", API_KEY).await
    }

    pub async fn introduced(&self) -> Device {
        let device = self.linked().await;

        self.request(
            Method::GET,
            &device.at("/v1/initialization"),
            &[("x-kobo-deviceid", DEVICE_HARDWARE_ID)],
        )
        .await;

        device
    }

    pub async fn send(&self, request: Request<Body>) -> Response<Body> {
        router().oneshot(request).await.expect("The router failed")
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        self.request(Method::GET, uri, &[]).await
    }

    pub async fn request(&self, method: Method, uri: &str, headers: &[(&str, &str)]) -> Response<Body> {
        let mut request = Request::builder().method(method).uri(uri).header("Host", HOST);

        for (name, value) in headers {
            request = request.header(*name, *value);
        }

        let request = request.body(Body::empty()).expect("Failed to build the request");

        self.send(request).await
    }

    pub async fn json(&self, method: Method, uri: &str, body: Value) -> Response<Body> {
        self.json_with(method, uri, &[], body).await
    }

    pub async fn json_with(
        &self,
        method: Method,
        uri: &str,
        headers: &[(&str, &str)],
        body: Value,
    ) -> Response<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Host", HOST)
            .header("Content-Type", "application/json");

        for (name, value) in headers {
            request = request.header(*name, *value);
        }

        let request = request
            .body(Body::from(body.to_string()))
            .expect("Failed to build the request");

        self.send(request).await
    }
}

pub struct Device {
    pub device_id: String,
    pub lookup_key: String,
}

impl Device {
    pub fn at(&self, path: &str) -> String {
        format!("/{}{path}", self.lookup_key)
    }
}

pub async fn body_json(response: Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read the body");

    serde_json::from_slice(&bytes).expect("The body was not JSON")
}

pub async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read the body")
        .to_vec()
}

async fn reset_database() {
    let triggers: Vec<String> = sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'trigger'")
        .fetch_all(database::pool())
        .await
        .expect("Failed to list the test triggers");

    for trigger in triggers {
        sqlx::query(&format!("DROP TRIGGER {trigger}"))
            .execute(database::pool())
            .await
            .expect("Failed to drop a test trigger");
    }

    let tables: Vec<String> = sqlx::query_scalar(
        r"
        SELECT name
        FROM sqlite_master
        WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations'
        ",
    )
    .fetch_all(database::pool())
    .await
    .expect("Failed to list the test tables");

    for table in tables {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(database::pool())
            .await
            .expect("Failed to empty a test table");
    }
}

pub async fn assert_internal_error(response: Response<Body>) {
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        body_json(response).await,
        json!({ "error_code": "InternalError", "message": "Internal error" })
    );
}

pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|_| panic!("Failed to read the fixture {name}"))
}

pub const EVERY_CAPABILITY: [&str; 4] = ["Read", "Create", "Update", "Delete"];

const SIGNING_KEY: &str = concat!(
    "MIIEowIBAAKCAQEAqW5moAsjTnf9nkk9pkbd1Ffwlo4NNTlrFJwwBqi5m6XIdUytD2Xlb7lty27HkBmI5JT+e5JGgFTYoW0z",
    "ZqbpvOKykMbwCpIwYTHYmKPlR78nycK4WfBfc9ofgaHx3dlm8qYIomVtVZqXByf187Luzqoh2T51zGFiNzP2jekB/waogRJv",
    "SpGNNhUs5DeABHjkHxg3vUO7kzl5K32fRtHWN07UT24dA4qRngp5MTmGXkicjK3yHiLm3rDQJW3HaB/SOORlC4t/yCXRmH+v",
    "0vYpCWfbh6Eu7/6ca/6oheYruiVW1kqVdTY69szrlBlhj+8NcW4AOEwgP3MLkPl+kFUpLQIDAQABAoIBAFJ+W19PLPiWuZho",
    "5qhf1r/9tTlInquscjONvGBxpDVhaZGbLKPTo/ta2Fd1b5r8o8dPt/wog0UWiodGaQFxAVsjMXgGNHehKAcawu/G1Oqy6sd0",
    "lFfODluU90Qhumae5F3+czPGmI5Wf3RPg5QNKfLbqTFiFO7W94ATP9dmfL9TDbI76329CCvm1ipNGl4n27ruDxyIAib3hbFf",
    "l7VFWUBrJTh8bJl0WeHniMr1Aov1Gbsb1pdEQsBhDr07fYlylvQOciqMWiC5Z52tfCKa2wupWJnmkvxx2siXcOywAZ0sSK0x",
    "kVuTOGIVdrXHzAkR5aSpob+xE/QeI5dQk8hKN00CgYEA5vG54ZjZalIMqYX4ZGLS4RvSr171EaPAjUfQmruACcRO4omqIONp",
    "4s0/LTzmtCYz0RXZI6k9y7JMqn4ThL6RiXbu96hWA1XqGI65IilN0KbQhQQa/sWOJFN8RVu2HKSCt9zSy8VU8Bdhr51uH08r",
    "KCx4WpE8F6eZjwvDR38KLVMCgYEAu9A0luQfV65w11+6hHzQXXjCaFAAa3pfu4dWx6TlU3N0Mqn8Q6RP3MKdTehziszb8Iky",
    "7rtHwSkKCKxI5qAGD8ZBdztIgsSAw7EbTMP02obubnfJDyjmdXXu5lpTPpw4AcuyIBVlZ41GRlfWVbOMVPVhCYSbJN0uljSt",
    "atTx/38CgYEAzdGqjpLxWN7s5/w8pgKo2/Q27RJcqK7ewUq6b9wgvROWFjEITS0/xeCTqFZ5aR4O2g11qmF/cFVkHCImdQx9",
    "mhxD1rXQikJ8UgyIlBhpgEXa2mERSfN4vYkl3I5im95FnWUQ/IfOLfc4mRFd/ktJNBGF5XlAK/izUxfxnntq738CgYB/lWik",
    "OEOyZKXzKOyjo8ZIOQiXpAElAfSWh9HDvZirn/qHkEH2EWDPLsV9bzNOTuv58rnOwxpzdVWDnsXHJV34yU0fmf4gFy0BwwYw",
    "l811xcOi92x2B+rGUF8BzMpP9f91+NIASYihV68tie3oOhhSfn504Mgjur0y/IXx1MawMQKBgDFUna3qP7iF0XU3+rSH3GLP",
    "aBRRIYQJizWsaq/TTG3jlkNxow4PmBXSQDf5YladLIB4+4ceGekQJOVH7Pw9NVyq9GtmIPGbaWhYdRfpwj/tDqO7o87GVRbh",
    "P3q0Y117QcLTQadztMTk+8alyhBOXGWdiC+wJxP+z1+nEYZ0oADf",
);

const SIGNING_KEY_MODULUS: &str = concat!(
    "qW5moAsjTnf9nkk9pkbd1Ffwlo4NNTlrFJwwBqi5m6XIdUytD2Xlb7lty27HkBmI5JT-e5JGgFTYoW0zZqbpvOKykMbwCpIw",
    "YTHYmKPlR78nycK4WfBfc9ofgaHx3dlm8qYIomVtVZqXByf187Luzqoh2T51zGFiNzP2jekB_waogRJvSpGNNhUs5DeABHjk",
    "Hxg3vUO7kzl5K32fRtHWN07UT24dA4qRngp5MTmGXkicjK3yHiLm3rDQJW3HaB_SOORlC4t_yCXRmH-v0vYpCWfbh6Eu7_6c",
    "a_6oheYruiVW1kqVdTY69szrlBlhj-8NcW4AOEwgP3MLkPl-kFUpLQ",
);

const SIGNING_KEY_ID: &str = "test-key";

fn signing_keys() -> JwkSet {
    serde_json::from_value(json!({ "keys": [{
        "kty": "RSA",
        "alg": "RS256",
        "use": "sig",
        "kid": SIGNING_KEY_ID,
        "n": SIGNING_KEY_MODULUS,
        "e": "AQAB",
    }] }))
    .expect("Failed to build the signing keys")
}

pub fn jwt(user_id: &str, is_admin: bool, capabilities: &[&str]) -> String {
    sign(user_id, is_admin, capabilities, 900)
}

pub fn expired_jwt(user_id: &str) -> String {
    sign(user_id, false, &EVERY_CAPABILITY, -900)
}

pub fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

fn sign(user_id: &str, is_admin: bool, capabilities: &[&str], expires_in: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs();

    let role = if is_admin {
        ProsaRole::Admin(user_id.to_owned())
    } else {
        ProsaRole::User(user_id.to_owned())
    };

    let claims = ProsaJWTClaims {
        role,
        capabilities: capabilities
            .iter()
            .map(|capability| (*capability).to_owned())
            .collect(),
        exp: now.saturating_add_signed(expires_in),
        session_id: "session".to_owned(),
        iss: PROSA_ISSUER.to_owned(),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(SIGNING_KEY_ID.to_owned());

    let key = BASE64_STANDARD
        .decode(SIGNING_KEY)
        .expect("Failed to decode the signing key");
    let token = jsonwebtoken::encode(&header, &claims, &EncodingKey::from_rsa_der(&key))
        .expect("Failed to sign the token");

    BASE64_STANDARD.encode(token)
}
