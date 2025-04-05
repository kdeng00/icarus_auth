use argon2::{
    Argon2, // The Argon2 algorithm struct
    PasswordVerifier,
    password_hash::{
        PasswordHasher,
        SaltString,
        rand_core::OsRng, // Secure random number generator
    },
};

pub fn hash_password(password: &String) -> Result<String, argon2::password_hash::Error> {
    let password_bytes = password.as_bytes();

    // Generate a random salt
    // SaltString::generate uses OsRng internally for cryptographic security
    let salt = SaltString::generate(&mut OsRng);

    // Create an Argon2 instance with default parameters (recommended)
    // You could customize parameters here if needed, but defaults are strong
    let argon2 = Argon2::default();

    // Hash the password with the salt
    // The output is a PasswordHash string format that includes algorithm, version,
    // parameters, salt, and the hash itself.
    let password_hash = argon2.hash_password(password_bytes, &salt)?.to_string();

    Ok(password_hash)
}

pub fn verify_password(
    password_attempt: &String,
    stored_hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let password_bytes = password_attempt.as_bytes();

    // Parse the stored hash string
    // This extracts the salt, parameters, and hash digest
    let parsed_hash = argon2::PasswordHash::new(stored_hash.as_str())?;

    // Create an Argon2 instance (it will use the parameters from the parsed hash)
    let argon2 = Argon2::default();

    // Verify the password against the parsed hash
    // This automatically uses the correct salt and parameters embedded in `parsed_hash`
    match argon2.verify_password(password_bytes, &parsed_hash) {
        Ok(()) => Ok(true),                                       // Passwords match
        Err(argon2::password_hash::Error::Password) => Ok(false), // Passwords don't match
        Err(e) => Err(e), // Some other error occurred (e.g., invalid hash format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let some_password = String::from("somethingrandom");
        match hash_password(&some_password) {
            Ok(p) => match verify_password(&some_password, p.clone()) {
                Ok(res) => {
                    assert_eq!(res, true);
                }
                Err(err) => {
                    assert!(false, "Error: {:?}", err.to_string());
                }
            },
            Err(eerr) => {
                assert!(false, "Error: {:?}", eerr.to_string());
            }
        }
    }
}
