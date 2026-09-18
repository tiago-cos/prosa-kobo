use super::models::{
    AuthError, JWTClaims, OAUTH_CONFIGS, OAUTH_TOKEN, PROSA_ISSUER, ProsaJWTClaims, ProsaToken,
};
use crate::{
    CONFIG,
    client::prosa::{Client, ProsaApi},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, jwk::JwkSet};
use rand::{TryRngCore, rngs::OsRng};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

static ENCODING_KEY: LazyLock<EncodingKey> =
    LazyLock::new(|| EncodingKey::from_secret(&load_or_generate_jwt_secret()));

static DECODING_KEY: LazyLock<DecodingKey> =
    LazyLock::new(|| DecodingKey::from_secret(&load_or_generate_jwt_secret()));

static PROSA_KEYS: LazyLock<HashMap<String, DecodingKey>> = LazyLock::new(|| {
    let client = Client::new(&CONFIG.prosa.scheme, &CONFIG.prosa.host, CONFIG.prosa.port);

    let jwks = client.jwks().expect("Failed to fetch Prosa signing keys");

    decoding_keys(&jwks)
});

pub fn generate_jwt(device_id: &str, duration: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs();

    let claims = JWTClaims {
        device_id: device_id.to_string(),
        exp: now + duration,
    };

    let token =
        jsonwebtoken::encode(&Header::default(), &claims, &ENCODING_KEY).expect("Failed to encode token");

    BASE64_STANDARD.encode(token)
}

pub fn verify_jwt(token: &str) -> Result<String, AuthError> {
    let token = BASE64_STANDARD.decode(token).or(Err(AuthError::InvalidToken))?;
    let token = String::from_utf8(token).or(Err(AuthError::InvalidToken))?;
    let validation = Validation::default();
    let token = jsonwebtoken::decode::<JWTClaims>(&token, &DECODING_KEY, &validation)?;

    Ok(token.claims.device_id)
}

pub fn verify_prosa_jwt(token: &str) -> Result<ProsaToken, AuthError> {
    decode_prosa_jwt(&PROSA_KEYS, token)
}

fn decode_prosa_jwt(keys: &HashMap<String, DecodingKey>, token: &str) -> Result<ProsaToken, AuthError> {
    let token = BASE64_STANDARD.decode(token).or(Err(AuthError::InvalidToken))?;
    let token = String::from_utf8(token).or(Err(AuthError::InvalidToken))?;

    let key_id = jsonwebtoken::decode_header(&token)?
        .kid
        .ok_or(AuthError::InvalidToken)?;

    let key = keys.get(&key_id).ok_or(AuthError::InvalidSignature)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[PROSA_ISSUER]);

    let token = jsonwebtoken::decode::<ProsaJWTClaims>(&token, key, &validation)?;

    Ok(token.claims.into())
}

fn decoding_keys(jwks: &JwkSet) -> HashMap<String, DecodingKey> {
    jwks.keys
        .iter()
        .filter_map(|jwk| {
            let key_id = jwk.common.key_id.clone()?;
            let key = DecodingKey::from_jwk(jwk).ok()?;

            Some((key_id, key))
        })
        .collect()
}

pub fn generate_oauth_config(host: &str, device_id: &str) -> Value {
    let json_string = OAUTH_CONFIGS
        .replace("{host}", host)
        .replace("{device_id}", device_id);

    serde_json::from_str(&json_string).expect("Failed to parse JSON")
}

pub fn generate_oauth_token(jwt_token: &str, jwt_duration: u64) -> Value {
    let json_string = OAUTH_TOKEN
        .replace("{jwt_token}", jwt_token)
        .replace("{jwt_duration}", &jwt_duration.to_string());

    serde_json::from_str(&json_string).expect("Failed to parse JSON")
}

fn load_or_generate_jwt_secret() -> Vec<u8> {
    let path = Path::new(&CONFIG.auth.jwt_key_path);

    if path.exists() {
        return fs::read(path).expect("Failed to read JWT secret");
    }

    let mut key = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut key)
        .expect("Failed to generate JWT secret");

    fs::write(path, key).expect("Failed to write JWT secret");

    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::authentication::models::ProsaRole;
    use jsonwebtoken::jwk::Jwk;

    const TEST_PRIVATE_KEY: &str = concat!(
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

    const TEST_MODULUS: &str = concat!(
        "qW5moAsjTnf9nkk9pkbd1Ffwlo4NNTlrFJwwBqi5m6XIdUytD2Xlb7lty27HkBmI5JT-e5JGgFTYoW0zZqbpvOKykMbwCpIw",
        "YTHYmKPlR78nycK4WfBfc9ofgaHx3dlm8qYIomVtVZqXByf187Luzqoh2T51zGFiNzP2jekB_waogRJvSpGNNhUs5DeABHjk",
        "Hxg3vUO7kzl5K32fRtHWN07UT24dA4qRngp5MTmGXkicjK3yHiLm3rDQJW3HaB_SOORlC4t_yCXRmH-v0vYpCWfbh6Eu7_6c",
        "a_6oheYruiVW1kqVdTY69szrlBlhj-8NcW4AOEwgP3MLkPl-kFUpLQ",
    );

    const TEST_KEY_ID: &str = "test-key-1";

    fn signing_key() -> EncodingKey {
        let der = BASE64_STANDARD
            .decode(TEST_PRIVATE_KEY)
            .expect("Failed to decode test key");

        EncodingKey::from_rsa_der(&der)
    }

    fn published_keys() -> HashMap<String, DecodingKey> {
        let jwk = format!(
            r#"{{"kty":"RSA","alg":"RS256","use":"sig","kid":"{TEST_KEY_ID}","n":"{TEST_MODULUS}","e":"AQAB"}}"#
        );

        let jwk: Jwk = serde_json::from_str(&jwk).expect("Failed to parse test JWK");

        decoding_keys(&JwkSet { keys: vec![jwk] })
    }

    fn claims(expires_in: i64, issuer: &str) -> ProsaJWTClaims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_secs();

        ProsaJWTClaims {
            role: ProsaRole::User("a-user".to_owned()),
            capabilities: vec!["Read".to_owned(), "Create".to_owned()],
            exp: now.saturating_add_signed(expires_in),
            session_id: "a-session".to_owned(),
            iss: issuer.to_owned(),
        }
    }

    fn sign(claims: &ProsaJWTClaims, key_id: &str) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(key_id.to_owned());

        let token = jsonwebtoken::encode(&header, claims, &signing_key()).expect("Failed to sign test token");

        BASE64_STANDARD.encode(token)
    }

    #[test]
    fn accepts_a_token_prosa_signed() {
        let token = sign(&claims(900, PROSA_ISSUER), TEST_KEY_ID);

        let token = decode_prosa_jwt(&published_keys(), &token).expect("Failed to verify token");

        assert_eq!(
            token,
            ProsaToken {
                user_id: "a-user".to_owned(),
                is_admin: false,
                capabilities: vec!["Read".to_owned(), "Create".to_owned()],
                session_id: "a-session".to_owned(),
            }
        );
    }

    #[test]
    fn reads_the_admin_role() {
        let mut claims = claims(900, PROSA_ISSUER);
        claims.role = ProsaRole::Admin("an-admin".to_owned());

        let token = sign(&claims, TEST_KEY_ID);
        let token = decode_prosa_jwt(&published_keys(), &token).expect("Failed to verify token");

        assert_eq!(token.user_id, "an-admin");
        assert!(token.is_admin);
    }

    #[test]
    fn rejects_an_expired_token() {
        let token = sign(&claims(-900, PROSA_ISSUER), TEST_KEY_ID);

        let error = decode_prosa_jwt(&published_keys(), &token);

        assert_eq!(error, Err(AuthError::ExpiredToken));
    }

    #[test]
    fn rejects_a_token_signed_by_a_key_prosa_does_not_publish() {
        let token = sign(&claims(900, PROSA_ISSUER), "some-other-key");

        let error = decode_prosa_jwt(&published_keys(), &token);

        assert_eq!(error, Err(AuthError::InvalidSignature));
    }

    #[test]
    fn rejects_a_token_from_another_issuer() {
        let token = sign(&claims(900, "not-prosa"), TEST_KEY_ID);

        let error = decode_prosa_jwt(&published_keys(), &token);

        assert_eq!(error, Err(AuthError::InvalidToken));
    }

    #[test]
    fn rejects_a_token_that_is_not_base64() {
        let error = decode_prosa_jwt(&published_keys(), "not a token");

        assert_eq!(error, Err(AuthError::InvalidToken));
    }

    #[test]
    fn ignores_a_published_key_without_an_id() {
        let jwk = format!(r#"{{"kty":"RSA","alg":"RS256","use":"sig","n":"{TEST_MODULUS}","e":"AQAB"}}"#);

        let jwk: Jwk = serde_json::from_str(&jwk).expect("Failed to parse test JWK");

        assert!(decoding_keys(&JwkSet { keys: vec![jwk] }).is_empty());
    }
}
