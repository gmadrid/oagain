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

pub mod preset;

use crate::consumer::builder::preset::Preset;
use crate::consumer::Consumer;
use crate::nonce_provider::{BasicNonce, NonceProvider};
use crate::Result;
use crate::{BasicConsumer, OagainError};
use url::Url;

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

    pub fn set_request_token_url(mut self, url: impl Into<Url>) -> Self {
        self.request_token_url = Some(url.into());
        self
    }

    pub fn set_user_authorization_url(mut self, url: impl Into<Url>) -> Self {
        self.user_authorization_url = Some(url.into());
        self
    }

    pub fn set_access_token_url(mut self, url: impl Into<Url>) -> Self {
        self.access_token_url = Some(url.into());
        self
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
}
