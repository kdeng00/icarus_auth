use josekit::{
    self,
    jws::alg::hmac::HmacJwsAlgorithm::Hs256,
    jwt::{self},
};

use time;

pub const KEY_ENV: &str = "SECRET_KEY";
pub const MESSAGE: &str = "Something random";
pub const ISSUER: &str = "icarus_auth";
pub const AUDIENCE: &str = "icarus";

pub fn get_issued() -> time::Result<time::OffsetDateTime> {
    Ok(time::OffsetDateTime::now_utc())
}

pub fn get_expiration(issued: &time::OffsetDateTime) -> Result<time::OffsetDateTime, time::Error> {
    let duration_expire = time::Duration::hours(4);
    Ok(*issued + duration_expire)
}

pub fn create_token(provided_key: &String) -> Result<(String, i64), josekit::JoseError> {
    let resource = icarus_models::token::TokenResource {
        message: String::from(MESSAGE),
        issuer: String::from(ISSUER),
        audiences: vec![String::from(AUDIENCE)],
    };
    icarus_models::token::create_token(provided_key, &resource, time::Duration::hours(4))
}

pub fn create_service_token(provided: &String) -> Result<(String, i64), josekit::JoseError> {
    let resource = icarus_models::token::TokenResource {
        message: String::from("Service random"),
        issuer: String::from(ISSUER),
        audiences: vec![String::from(AUDIENCE)],
    };
    icarus_models::token::create_token(provided, &resource, time::Duration::hours(1))
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
        let rt = tokio::runtime::Runtime::new().unwrap();
        let special_key = rt.block_on(icarus_envy::environment::get_secret_key());
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
