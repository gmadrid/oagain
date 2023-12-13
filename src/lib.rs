use thiserror::Error;

mod config;
mod consumer;
mod nonce_provider;
mod parameters;
mod pencoding;

#[cfg(test)]
mod test_constants;

/// Error type for the OAgain library.
#[derive(Error, Debug)]
pub enum OagainError {
    #[error("A param starting with 'oauth_' is disallowed, {0}")]
    DisallowedOauthParam(String),

    #[error("A required authority is missing from a URL, {0}.")]
    MissingAuthority(String),

    #[error("A required path is missing from a URL, {0}.")]
    MissingPath(String),

    #[error("A required scheme is missing from a URL, {0}.")]
    MissingScheme(String),

    #[error("A parse error occurred in a URL.")]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, OagainError>;
