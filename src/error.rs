use thiserror::Error;

/// Error type for the OAgain library.
#[derive(Error, Debug)]
pub enum OagainError {
    // TODO: I would like to figure out how to include the underlying error in this.
    #[error("A bad URL was passed to Consumer Builder")]
    BadUrl,

    #[error("A parse error occurred in a date string.")]
    ChronoError(#[from] chrono::ParseError),

    #[error("A param starting with 'oauth_' is disallowed, {0}")]
    DisallowedOauthParam(String),

    #[error("An IO error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("The access token was missing from the server response")]
    MissingAccessToken,

    #[error("The access token url was missing while creating the Consumer")]
    MissingAccessTokenUrl,

    #[error("A required authority is missing from a URL, {0}.")]
    MissingAuthority(String),

    #[error("The user authorization URL is not specified while creating the Consumer.")]
    MissingAuthUrl,

    #[error("The consumer secret was not found in the secrets file.")]
    MissingConsumerSecret(&'static str),

    #[error("The consumer token was not found ({0}).")]
    MissingConsumerToken(&'static str),

    #[error("A required path is missing from a URL, {0}.")]
    MissingPath(String),

    #[error("The request token is missing. Perhaps you're calling stuff out of order.")]
    MissingRequestToken,

    #[error("The request token url is missing.")]
    MissingRequestTokenUrl,

    #[error("A required scheme is missing from a URL, {0}.")]
    MissingScheme(String),

    #[error("The token save time is not stored in the save file.")]
    MissingTokenSaveTime,

    #[error("A required token secret was not found")]
    MissingTokenSecret,

    // TODO: Ideally, we should try to renew it if it's eligible.
    //       (See the [docs](https://apisb.etrade.com/docs/api/authorization/renew_access_token.html).
    // TODO: Also, we should save the _last use_ time, not the save time.
    #[error("The token was too old to use.")]
    OldToken,

    #[error("A reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("An error occurred while reading the toml file: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("A parse error occurred in a URL.")]
    UrlParseError(#[from] url::ParseError),

    #[error("An error occurred converting a String to Utf-8")]
    Utf3Error(#[from] std::string::FromUtf8Error),
}

/// Result type for the OAgain library.
pub type Result<T> = std::result::Result<T, OagainError>;
