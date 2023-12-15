use thiserror::Error;

mod config;
mod constants;
mod consumer;
mod nonce_provider;
mod parameters;
mod pencoding;
mod signing;

#[cfg(test)]
mod test_constants;

/// Error type for the OAgain library.
#[derive(Error, Debug)]
pub enum OagainError {
    #[error("A param starting with 'oauth_' is disallowed, {0}")]
    DisallowedOauthParam(String),

    #[error("An IO error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("A required authority is missing from a URL, {0}.")]
    MissingAuthority(String),

    #[error("The consumer secret was not found in the secrets file.")]
    MissingConsumerSecret,

    #[error("The consumer token was not found in the secrets file.")]
    MissingConsumerToken,

    #[error("A required path is missing from a URL, {0}.")]
    MissingPath(String),

    #[error("A required scheme is missing from a URL, {0}.")]
    MissingScheme(String),

    #[error("A reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("An error occurred while reading the toml file: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("A parse error occurred in a URL.")]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, OagainError>;

pub use config::Config;
pub use consumer::BasicConsumer;
