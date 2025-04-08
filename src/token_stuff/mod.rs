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

pub fn get_issued() -> time::Result<time::OffsetDateTime> {
    Ok(time::OffsetDateTime::now_utc())
}

pub fn get_expiration(issued: &time::OffsetDateTime) -> Result<time::OffsetDateTime, time::Error> {
    let duration_expire = time::Duration::hours(4);
    Ok(*issued + duration_expire)
}

mod util {
    pub fn time_to_std_time(
        provided_time: &time::OffsetDateTime,
    ) -> Result<std::time::SystemTime, std::time::SystemTimeError> {
        let converted = std::time::SystemTime::from(*provided_time);
        Ok(converted)
    }
}

pub fn create_token(provided_key: &String) -> Result<(String, i64), josekit::JoseError> {
    let mut header = JwsHeader::new();
    header.set_token_type(TOKENTYPE);

    let mut payload = JwtPayload::new();
    payload.set_subject(MESSAGE);
    payload.set_issuer(ISSUER);
    payload.set_audience(vec![AUDIENCE]);
    match get_issued() {
        Ok(issued) => {
            let expire = get_expiration(&issued).unwrap();
            payload.set_issued_at(&util::time_to_std_time(&issued).unwrap());
            payload.set_expires_at(&util::time_to_std_time(&expire).unwrap());

            let key: String = if provided_key.is_empty() {
                get_key().unwrap()
            } else {
                provided_key.to_owned()
            };

            let signer = Hs256.signer_from_bytes(key.as_bytes()).unwrap();
            Ok((
                josekit::jwt::encode_with_signer(&payload, &header, &signer).unwrap(),
                (expire - time::OffsetDateTime::UNIX_EPOCH).whole_seconds(),
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
