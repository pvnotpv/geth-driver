use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac, KeyInit};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use rand::Rng;

pub fn generate_jwt(jwt_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret = fs::read_to_string(jwt_file)?;
    let secret_hex = secret.trim().strip_prefix("0x").unwrap_or(secret.trim());

    let secret_bytes = hex::decode(secret_hex)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let header = URL_SAFE_NO_PAD.encode(r#"{"typ":"JWT","alg":"HS256"}"#);

    let payload = URL_SAFE_NO_PAD.encode(
        format!(r#"{{"iat":{}}}"#, now)
    );

    let unsigned = format!("{}.{}", header, payload);

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)?;
    mac.update(unsigned.as_bytes());

    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    Ok(format!("{}.{}", unsigned, signature))
}



pub fn generate_secret() -> String {
    let mut bytes = [0u8; 32];

    rand::rng().fill_bytes(&mut bytes);

    hex::encode(bytes)
}

