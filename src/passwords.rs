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
        let needs_rehash = ok
            && !(parsed.algorithm == Algorithm::Argon2id.ident()
                && parsed.version == Some(Version::V0x13.into())
                && parsed.params.m_cost == Some(self.a2.params().m_cost())
                && parsed.params.t_cost == Some(self.a2.params().t_cost())
                && parsed.params.p_cost == Some(self.a2.params().p_cost()));
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

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let phc = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashingFailed(e.to_string()))?
        .to_string();
    Ok(phc)
}

pub fn verify_password(password: &str, pw_hash: &str) -> Result<bool> {
    let parsed =
        PasswordHash::new(pw_hash).map_err(|e| PasswordError::InvalidHash(e.to_string()))?;
    let argon2 = Argon2::default();
    let password_ok = argon2.verify_password(password.as_bytes(), &parsed).is_ok();
    Ok(password_ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_ok() {
        let h = hash_password("secret").unwrap();
        assert!(verify_password("secret", &h).unwrap());
    }

    #[test]
    fn test_invalid_hash() {
        let result = verify_password("secret", "invalid_hash");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PasswordError::InvalidHash(_)));
    }
}
