//! Configuration handling for the application.
//!
//! For now we don't rely on external environment configuration, but this
//! module is structured so we can easily switch to reading real environment
//! variables (or even a .env / config file) later. The `Config::from_env`
//! method performs that loading with sensible development defaults.

use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Environment variable names. Keeping them public lets other crates (tests,
/// build scripts) refer to them if needed later.
pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
pub const ENV_BIND_ADDR: &str = "BIND_ADDR";
pub const ENV_JWT_SECRET: &str = "JWT_SECRET";

/// Default development values used when environment variables are absent.
const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/capsule";
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_JWT_SECRET: &str = "dev-secret-change-me";

/// Application runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    database_url: String,
    bind_addr: String,
    jwt_secret: String,
}

impl Config {
    /// Create a new config explicitly.
    pub fn new(
        database_url: impl Into<String>,
        bind_addr: impl Into<String>,
        jwt_secret: impl Into<String>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            bind_addr: bind_addr.into(),
            jwt_secret: jwt_secret.into(),
        }
    }

    /// Load from environment variables, falling back to development defaults.
    ///
    /// This never fails today because we only do simple string extraction.
    /// In the future, validation (e.g. parse addresses, minimum secret length)
    /// can cause it to return a `ConfigError`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            env::var(ENV_DATABASE_URL).unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let bind_addr = env::var(ENV_BIND_ADDR).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
        let jwt_secret =
            env::var(ENV_JWT_SECRET).unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());
        // Placeholder spot for future validation hooks.
        Ok(Self {
            database_url,
            bind_addr,
            jwt_secret,
        })
    }

    /// Database connection string (PostgreSQL URL).
    pub fn database_url(&self) -> &str {
        &self.database_url
    }
    /// TCP bind address (host:port) for the HTTP server.
    pub fn bind_addr(&self) -> &str {
        &self.bind_addr
    }
    /// Secret used for signing/verifying JWTs.
    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    /// Development defaults (mirrors `from_env` with no env overrides).
    pub fn default() -> Self {
        // not `Default` impl yet to keep explicit semantics
        Self::new(DEFAULT_DATABASE_URL, DEFAULT_BIND_ADDR, DEFAULT_JWT_SECRET)
    }
}

/// Errors that can occur while building a configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// Reserved for future validation failures.
    InvalidValue { field: &'static str, reason: String },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidValue { field, reason } => {
                write!(f, "invalid value for '{}': {}", field, reason)
            }
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex; // bring std::env into this module's scope explicitly

    // Ensure environment-variable manipulating tests run serially.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn clear_env() {
        for key in [ENV_DATABASE_URL, ENV_BIND_ADDR, ENV_JWT_SECRET] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    fn defaults_when_env_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        clear_env();
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.database_url(), super::DEFAULT_DATABASE_URL);
        assert_eq!(cfg.bind_addr(), super::DEFAULT_BIND_ADDR);
        assert_eq!(cfg.jwt_secret(), super::DEFAULT_JWT_SECRET);
    }

    #[test]
    fn overrides_when_env_present() {
        let _guard = ENV_MUTEX.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var(ENV_DATABASE_URL, "postgres://user:pw@db:5432/other");
            env::set_var(ENV_BIND_ADDR, "0.0.0.0:9000");
            env::set_var(ENV_JWT_SECRET, "super-secret");
        }
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.database_url(), "postgres://user:pw@db:5432/other");
        assert_eq!(cfg.bind_addr(), "0.0.0.0:9000");
        assert_eq!(cfg.jwt_secret(), "super-secret");
    }
}
