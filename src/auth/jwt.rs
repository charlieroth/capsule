use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiry timestamp
    pub iat: usize,  // Issued at timestamp
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    pub fn generate_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::default();
        validation.leeway = 60; // Allow 60 seconds clock skew

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let jwt_service = JwtService::new("test-secret");
        let user_id = Uuid::new_v4();

        let token = jwt_service.generate_token(user_id).unwrap();
        assert!(!token.is_empty());

        let claims = jwt_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > Utc::now().timestamp() as usize);
    }

    #[test]
    fn test_verify_invalid_token() {
        let jwt_service = JwtService::new("test-secret");
        let result = jwt_service.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let jwt_service1 = JwtService::new("secret-1");
        let jwt_service2 = JwtService::new("secret-2");
        let user_id = Uuid::new_v4();

        let token = jwt_service1.generate_token(user_id).unwrap();
        let result = jwt_service2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_expired_token() {
        let jwt_service = JwtService::new("test-secret");
        let user_id = Uuid::new_v4();

        let now = Utc::now();
        let expired_time = now - Duration::hours(25); // Expired 1 hour ago (token expires after 24h)

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expired_time.timestamp() as usize,
            iat: (expired_time - Duration::hours(24)).timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &jwt_service.encoding_key).unwrap();
        let result = jwt_service.verify_token(&token);
        assert!(result.is_err());
    }
}
