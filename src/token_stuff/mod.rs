use josekit::{
    self,
    jws::{JwsHeader, alg::hmac::HmacJwsAlgorithm::Hs256},
    jwt::{self, JwtPayload},
};

use time;

pub const TOKENTYPE: &str = "JWT";
pub const KEY_ENV: &str = "SECRET_KEY";
pub const MESSAGE: &str = "Something random";
pub const ISSUER: &str = "icarus_auth";
pub const AUDIENCE: &str = "icarus";

pub fn get_key() -> Result<String, dotenvy::Error> {
    dotenvy::dotenv().ok();
    let key = std::env::var(KEY_ENV).expect("SECRET_KEY_NOT_FOUND");
    Ok(key)
}

pub fn get_expiration() -> time::Result<time::Duration> {
    let now = time::OffsetDateTime::now_utc();
    let epoch = time::OffsetDateTime::UNIX_EPOCH;
    let since_the_epoch = now - epoch;
    Ok(since_the_epoch)
}

pub fn create_token(provided_key: &String) -> Result<(String, i64), josekit::JoseError> {
    let mut header = JwsHeader::new();
    header.set_token_type(TOKENTYPE);

    let mut payload = JwtPayload::new();
    payload.set_subject(MESSAGE);
    payload.set_issuer(ISSUER);
    payload.set_audience(vec![AUDIENCE]);
    match get_expiration() {
        Ok(duration) => {
            let expire = duration.whole_seconds();
            let _ = payload.set_claim(
                "expiration",
                Some(serde_json::to_value(expire.to_string()).unwrap()),
            );

            let key: String = if provided_key.is_empty() {
                get_key().unwrap()
            } else {
                provided_key.to_owned()
            };

            let signer = Hs256.signer_from_bytes(key.as_bytes()).unwrap();
            Ok((
                josekit::jwt::encode_with_signer(&payload, &header, &signer).unwrap(),
                duration.whole_seconds(),
            ))
        }
        Err(e) => Err(josekit::JoseError::InvalidClaim(e.into())),
    }
}

pub fn verify_token(key: &String, token: &String) -> bool {
    let ver = Hs256.verifier_from_bytes(key.as_bytes()).unwrap();
    let (payload, _header) = jwt::decode_with_verifier(token, &ver).unwrap();
    match payload.subject() {
        Some(_sub) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_tokenize() {
        let special_key = get_key().unwrap();
        match create_token(&special_key) {
            Ok((token, _duration)) => {
                let result = verify_token(&special_key, &token);
                assert!(result, "Token not verified");
            }
            Err(err) => {
                assert!(false, "Error: {:?}", err.to_string());
            }
        };
    }
}
