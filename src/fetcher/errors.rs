use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("dns failure: {0}")]
    Dns(String),

    #[error("tls error: {0}")]
    Tls(String),

    #[error("connect timeout")]
    ConnectTimeout,

    #[error("request timeout")]
    RequestTimeout,

    #[error("too many redirects")]
    RedirectLoop,

    #[error("http error {status}")]
    Http {
        status: reqwest::StatusCode,
        retriable: bool,
    },

    #[error("body too large ({0} bytes)")]
    BodyTooLarge(u64),

    #[error("unsupported content-type: {0}")]
    UnsupportedContentType(String),

    #[error("charset error: {0}")]
    Charset(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("unknown: {0}")]
    Unknown(String),
}

impl FetchError {
    pub fn should_retry(&self) -> bool {
        match self {
            // Fatal errors - don't retry
            Self::InvalidUrl(_) => false,
            Self::BodyTooLarge(_) => false,
            Self::UnsupportedContentType(_) => false,
            Self::Charset(_) => false,
            Self::Http { retriable, .. } => *retriable,

            // Temporary errors - retry
            Self::Dns(_) => true,
            Self::Tls(_) => true,
            Self::ConnectTimeout => true,
            Self::RequestTimeout => true,
            Self::RedirectLoop => true,
            Self::Io(_) => true,
            Self::Unknown(_) => true,
        }
    }

    pub fn from_reqwest_error(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            if err.is_connect() {
                Self::ConnectTimeout
            } else {
                Self::RequestTimeout
            }
        } else if err.is_redirect() {
            Self::RedirectLoop
        } else if let Some(status) = err.status() {
            Self::Http {
                status,
                retriable: status.is_server_error(),
            }
        } else if err.is_request() {
            // DNS, connection errors
            Self::Dns(err.to_string())
        } else {
            Self::Unknown(err.to_string())
        }
    }
}
