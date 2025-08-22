use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashingFailed(String),

    #[error("Failed to parse password hash: {0}")]
    InvalidHash(String),
}

pub type Result<T> = std::result::Result<T, PasswordError>;

#[derive(Clone)]
pub struct Passwords<'a> {
    a2: Argon2<'a>,
    min_len: usize,
    max_len: usize,
}

impl<'a> Passwords<'a> {
    pub fn new(mem_kib: u32, iters: u32, lanes: u32) -> Self {
        let params = Params::new(mem_kib, iters, lanes, None).expect("argon2 params");
        let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        Self {
            a2,
            min_len: 8,
            max_len: 512,
        }
    }

    pub fn hash(&self, password: &str) -> Result<String> {
        self.guard_length(password)?;
        let salt = SaltString::generate(&mut OsRng);
        let phc = self
            .a2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;
        Ok(phc.to_string())
    }

    pub fn verify(&self, password: &str, pw_hash: &str) -> Result<(bool, bool)> {
        let parsed =
            PasswordHash::new(pw_hash).map_err(|e| PasswordError::InvalidHash(e.to_string()))?;
        let ok = self
            .a2
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();
        let needs_rehash = if ok {
            match Params::try_from(&parsed) {
                Ok(parsed_params) => {
                    parsed.algorithm != Algorithm::Argon2id.ident()
                        || parsed.version != Some(Version::V0x13.into())
                        || parsed_params != *self.a2.params()
                }
                Err(_) => true,
            }
        } else {
            false
        };
        Ok((ok, needs_rehash))
    }

    fn guard_length(&self, s: &str) -> Result<()> {
        let len = s.chars().count();
        if len < self.min_len || len > self.max_len {
            return Err(PasswordError::HashingFailed(
                "password length out of bounds".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_hasher() -> Passwords<'static> {
        Passwords::new(65536, 2, 1)
    }

    #[test]
    fn test_hash_success() {
        let hasher = create_hasher();
        let password = "test_password";
        let result = hasher.hash(password);
        assert!(result.is_ok());
        
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$v=19$"));
    }

    #[test]
    fn test_verify_success() {
        let hasher = create_hasher();
        let password = "test_password";
        let hash = hasher.hash(password).unwrap();
        
        let result = hasher.verify(password, &hash);
        assert!(result.is_ok());
        
        let (is_valid, _needs_rehash) = result.unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_wrong_password() {
        let hasher = create_hasher();
        let password = "test_password";
        let wrong_password = "wrong_password";
        let hash = hasher.hash(password).unwrap();
        
        let result = hasher.verify(wrong_password, &hash);
        assert!(result.is_ok());
        
        let (is_valid, needs_rehash) = result.unwrap();
        assert!(!is_valid);
        assert!(!needs_rehash);
    }

    #[test]
    fn test_hash_password_too_short() {
        let hasher = create_hasher();
        let short_password = "1234567"; // 7 chars, min is 8
        
        let result = hasher.hash(short_password);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PasswordError::HashingFailed(msg) => {
                assert_eq!(msg, "password length out of bounds");
            }
            _ => panic!("Expected HashingFailed error"),
        }
    }

    #[test]
    fn test_hash_password_too_long() {
        let hasher = create_hasher();
        let long_password = "a".repeat(513); // 513 chars, max is 512
        
        let result = hasher.hash(&long_password);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PasswordError::HashingFailed(msg) => {
                assert_eq!(msg, "password length out of bounds");
            }
            _ => panic!("Expected HashingFailed error"),
        }
    }

    #[test]
    fn test_hash_password_boundary_lengths() {
        let hasher = create_hasher();
        
        // Test minimum length (8 chars)
        let min_password = "12345678";
        assert!(hasher.hash(min_password).is_ok());
        
        // Test maximum length (512 chars)
        let max_password = "a".repeat(512);
        assert!(hasher.hash(&max_password).is_ok());
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let hasher = create_hasher();
        let password = "test_password";
        let invalid_hash = "invalid_hash_format";
        
        let result = hasher.verify(password, invalid_hash);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PasswordError::InvalidHash(_) => {
                // Expected error
            }
            _ => panic!("Expected InvalidHash error"),
        }
    }

    #[test]
    fn test_verify_empty_hash() {
        let hasher = create_hasher();
        let password = "test_password";
        let empty_hash = "";
        
        let result = hasher.verify(password, empty_hash);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PasswordError::InvalidHash(_) => {
                // Expected error
            }
            _ => panic!("Expected InvalidHash error"),
        }
    }

    #[test]
    fn test_rehash_detection_different_params() {
        // Create hasher with different params
        let old_hasher = Passwords::new(32768, 1, 1); // Different memory
        let new_hasher = Passwords::new(65536, 2, 1); // Current params
        
        let password = "test_password";
        let old_hash = old_hasher.hash(password).unwrap();
        
        let result = new_hasher.verify(password, &old_hash);
        assert!(result.is_ok());
        
        let (is_valid, needs_rehash) = result.unwrap();
        assert!(is_valid);
        assert!(needs_rehash);
    }

    #[test]
    fn test_unicode_password() {
        let hasher = create_hasher();
        let unicode_password = "пароль123🔒";
        
        let hash_result = hasher.hash(unicode_password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        let verify_result = hasher.verify(unicode_password, &hash);
        assert!(verify_result.is_ok());
        
        let (is_valid, _needs_rehash) = verify_result.unwrap();
        assert!(is_valid);
    }
}
