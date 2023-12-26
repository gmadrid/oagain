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

use log::{info, warn};
use std::fs::{File, Permissions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use toml::Value;
use url::Url;

use crate::constants::{ACCESS_TOKEN_NAME, TOKEN_SECRET_NAME};
use crate::consumer::builder::preset::Preset;
use crate::consumer::state::ConsumerState;
use crate::consumer::state::ConsumerState::FullAuth;
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

    save_file: Option<PathBuf>,
    init_state: ConsumerState,
}

fn read_key_and_secret(path: impl AsRef<Path>) -> Result<(String, String)> {
    let mut s = String::new();
    let mut f = File::open(&path)?;
    if f.metadata()?.mode() & 0o777 != 0o600 {
        warn!(
            "File permissions on secrets file should be 0600: {}",
            path.as_ref().to_string_lossy()
        );
    }
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

fn read_access_key_and_secret(path: impl AsRef<Path>) -> Result<(String, String)> {
    let mut s = String::new();
    let mut f = File::open(&path)?;
    if f.metadata()?.mode() & 0o777 != 0o600 {
        warn!(
            "File permissions on save file should be 0600: {}",
            path.as_ref().to_string_lossy()
        );
    }
    f.read_to_string(&mut s)?;
    let table = s.parse::<toml::Table>()?;

    let Some(Value::String(key)) = table.get(ACCESS_TOKEN_NAME) else {
        return Err(OagainError::MissingAccessToken);
    };
    let Some(Value::String(secret)) = table.get(TOKEN_SECRET_NAME) else {
        return Err(OagainError::MissingTokenSecret);
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
            save_file: None,
            init_state: Default::default(),
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
            save_file: self.save_file,
            state: self.init_state,
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

    pub fn use_secrets_file(self, path: impl AsRef<Path>) -> Result<Self> {
        let (consumer_key, consumer_secret) = read_key_and_secret(path)?;
        Ok(self
            .set_consumer_key(consumer_key)
            .set_consumer_secret(consumer_secret))
    }

    pub fn use_save_file(mut self, path_in: impl AsRef<Path>) -> Result<Self> {
        let path = path_in.as_ref();
        self.save_file = Some(path.to_path_buf());

        if path.try_exists()? {
            if let Ok((access_token, token_secret)) = read_access_key_and_secret(path) {
                self.init_state = FullAuth {
                    access_token,
                    token_secret,
                }
            }
        } else {
            // not present, so create and protect.
            let f = File::create(path)?;
            f.set_permissions(Permissions::from_mode(0o600))?;
            info!("save file created at '{}'", path.to_string_lossy());
        }

        Ok(self)
    }
}
