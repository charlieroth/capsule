use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("Failed to compile email regex")
});

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

impl SignupRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !EMAIL_REGEX.is_match(&self.email) {
            return Err("Invalid email format".to_string());
        }
        if self.password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }
        if self.password.len() > 512 {
            return Err("Password too long".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !EMAIL_REGEX.is_match(&self.email) {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signup_request_valid_email() {
        let request = SignupRequest {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_signup_request_invalid_email() {
        let request = SignupRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_signup_request_password_too_short() {
        let request = SignupRequest {
            email: "user@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_login_request_valid() {
        let request = LoginRequest {
            email: "user@example.com".to_string(),
            password: "any_password".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
