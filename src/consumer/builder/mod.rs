// build_with_nonce_provider()
// build() - uses default nonce provider
//
// things to set:
// - request token url
// - user auth url
// - access token url
// - user auth key param name
// - user auth token param name
//
// - consumer key
// - consumer secret
//
// - preset

use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml::Value;
use url::Url;

use crate::consumer::builder::preset::Preset;
use crate::consumer::Consumer;
use crate::error::OagainError::BadUrl;
use crate::error::{OagainError, Result};
use crate::nonce_provider::{BasicNonce, NonceProvider};
use crate::BasicConsumer;

pub mod preset;

#[derive(Debug)]
pub struct Builder {
    request_token_url: Option<Url>,
    user_authorization_url: Option<Url>,
    access_token_url: Option<Url>,

    user_auth_key_param_name: String,
    user_auth_token_param_name: String,

    consumer_key: Option<String>,
    consumer_secret: Option<String>,
}

fn read_key_and_secret(path: impl AsRef<Path>) -> Result<(String, String)> {
    let mut s = String::new();
    let mut f = File::open(path)?;
    f.read_to_string(&mut s)?;
    let table = s.parse::<toml::Table>()?;

    let Some(Value::String(key)) = table.get("token") else {
        return Err(OagainError::MissingConsumerToken("in secrets file"));
    };
    let Some(Value::String(secret)) = table.get("secret") else {
        return Err(OagainError::MissingConsumerSecret("in secrets file"));
    };
    Ok((key.to_string(), secret.to_string()))
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            request_token_url: None,
            user_authorization_url: None,
            access_token_url: None,
            user_auth_key_param_name: "oauth_consumer_key".to_string(),
            user_auth_token_param_name: "oauth_token".to_string(),
            consumer_key: None,
            consumer_secret: None,
        }
    }
}

impl Builder {
    pub fn build_with_nonce_provider<NP: NonceProvider>(self, np: NP) -> Result<Consumer<NP>> {
        Ok(Consumer {
            consumer_key: self
                .consumer_key
                .ok_or(OagainError::MissingConsumerToken("in builder"))?,
            consumer_secret: self
                .consumer_secret
                .ok_or(OagainError::MissingConsumerSecret("in builder"))?,
            nonce_provider: np,
            request_token_url: self
                .request_token_url
                .ok_or(OagainError::MissingRequestTokenUrl)?,
            user_authorization_url: self
                .user_authorization_url
                .ok_or(OagainError::MissingAuthUrl)?,
            access_token_url: self
                .access_token_url
                .ok_or(OagainError::MissingAccessTokenUrl)?,
            user_auth_key_param_name: self.user_auth_key_param_name,
            user_auth_token_param_name: self.user_auth_token_param_name,
            state: Default::default(),
        })
    }

    pub fn build(self) -> Result<BasicConsumer> {
        self.build_with_nonce_provider(BasicNonce::default())
    }

    pub fn use_preset(self, preset: impl Preset) -> Result<Self> {
        preset.setup_builder(self)
    }

    pub fn set_request_token_url(mut self, url: impl TryInto<Url>) -> Result<Self> {
        self.request_token_url = Some(url.try_into().map_err(|_| BadUrl)?);
        Ok(self)
    }

    pub fn set_user_authorization_url(mut self, url: impl TryInto<Url>) -> Result<Self> {
        self.user_authorization_url = Some(url.try_into().map_err(|_| BadUrl)?);
        Ok(self)
    }

    pub fn set_access_token_url(mut self, url: impl TryInto<Url>) -> Result<Self> {
        self.access_token_url = Some(url.try_into().map_err(|_| BadUrl)?);
        Ok(self)
    }

    pub fn set_user_auth_key_param_name(mut self, val: impl Into<String>) -> Self {
        self.user_auth_key_param_name = val.into();
        self
    }

    pub fn set_user_auth_token_param_name(mut self, val: impl Into<String>) -> Self {
        self.user_auth_token_param_name = val.into();
        self
    }

    pub fn set_consumer_key(mut self, val: impl Into<String>) -> Self {
        self.consumer_key = Some(val.into());
        self
    }

    pub fn set_consumer_secret(mut self, val: impl Into<String>) -> Self {
        self.consumer_secret = Some(val.into());
        self
    }

    pub fn use_secrets_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let (consumer_key, consumer_secret) = read_key_and_secret(path)?;
        Ok(self
            .set_consumer_key(consumer_key)
            .set_consumer_secret(consumer_secret))
    }
}
