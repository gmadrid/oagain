use std::convert::TryFrom;

use url::Url;

use crate::constants::{OAUTH_CONSUMER_KEY_PARAM_NAME, OAUTH_TOKEN_PARAM_NAME};
use crate::{OagainError, Result};

/// Request URLs as defined in Spec 4.1.
#[derive(Debug)]
pub struct Config {
    pub request_token_url: Url,
    pub user_authorization_url: Url,
    pub access_token_url: Url,

    pub user_auth_key_param_name: String,
    pub user_auth_token_param_name: String,
}

impl Config {
    pub fn new(
        request_token_url: impl AsRef<str>,
        user_authorization_url: impl AsRef<str>,
        access_token_url: impl AsRef<str>,
    ) -> Result<Config> {
        let config = Config {
            request_token_url: Url::try_from(request_token_url.as_ref())?,
            user_authorization_url: Url::try_from(user_authorization_url.as_ref())?,
            access_token_url: Url::try_from(access_token_url.as_ref())?,

            user_auth_key_param_name: OAUTH_CONSUMER_KEY_PARAM_NAME.into(),
            user_auth_token_param_name: OAUTH_TOKEN_PARAM_NAME.into(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn set_user_auth_param_names(
        mut self,
        key_name: impl AsRef<str>,
        token_name: impl AsRef<str>,
    ) -> Self {
        self.user_auth_key_param_name = key_name.as_ref().into();
        self.user_auth_token_param_name = token_name.as_ref().into();
        self
    }

    /// Check that we meet the requirements in Spec 4.1
    fn validate(&self) -> Result<()> {
        Config::validate_url(&self.request_token_url)?;
        Config::validate_url(&self.user_authorization_url)?;
        Config::validate_url(&self.access_token_url)?;
        Ok(())
    }

    fn validate_url(url: &Url) -> Result<()> {
        // MUST include scheme, authority and_path.
        // MUST NOT include any oauth parameters (params beginning with 'oauth_')
        if url.scheme().is_empty() {
            return Err(OagainError::MissingScheme(url.to_string()));
        }
        if !url.has_authority() {
            return Err(OagainError::MissingAuthority(url.to_string()));
        }
        if url.path().is_empty() {
            return Err(OagainError::MissingPath(url.to_string()));
        }
        if url.query_pairs().any(|(n, _v)| n.starts_with("oauth_")) {
            return Err(OagainError::DisallowedOauthParam(url.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid() {
        assert!(Config::new(
            "https://photos.example.net/request_token",
            "http://photos.example.net/authorize",
            "https://photos.example.net/access_token",
        )
        .is_ok());
    }

    #[test]
    fn invalid_scheme() {
        assert!(Config::new(
            "//photos.example.net/request_token",
            "http://photos.example.net/authorize",
            "https://photos.example.net/access_token",
        )
        .is_err());
    }
    #[test]
    fn invalid_authority() {
        assert!(Config::new(
            "https://photos.example.net/request_token",
            "data:foo/bar/baz",
            "https://photos.example.net/access_token",
        )
        .is_err());
    }

    #[test]
    fn invalid_path() {
        assert!(Config::new(
            "https://photos.example.net/request_token",
            "http://photos.example.net/authorize",
            "data:",
        )
        .is_err());
    }

    #[test]
    fn invalid_has_oauth_param() {
        assert!(Config::new(
            "https://photos.example.net/request_token?oauth_token=foobar",
            "http://photos.example.net/authorize",
            "https://photos.example.net/access_token",
        )
        .is_err());
    }

    #[test]
    fn valid_has_non_oauth_param() {
        assert!(Config::new(
            "https://photos.example.net/request_token?auth_token=foobar",
            "http://photos.example.net/authorize",
            "https://photos.example.net/access_token",
        )
        .is_ok());
    }
}
