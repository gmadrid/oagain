use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::path::PathBuf;

use chrono::{Datelike, Timelike, Utc};
use itertools::Itertools;
use log::{debug, error};
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use toml::Value;
use url::Url;

pub use builder::preset::ETradePreset;
use state::ConsumerState;

use crate::constants::*;
pub use crate::consumer::builder::Builder;
use crate::error::{OagainError, Result};
use crate::nonce_provider::{BasicNonce, NonceProvider, SystemEpochProvider};
use crate::parameters::{decode_params_string, ParamPair};
use crate::signing::{concat_request_elements, make_signing_key, sign_string_hmac};
use crate::util::BoolToOption;

mod builder;
mod state;

/// A basic consumer that uses the standard time-based nonce provider.
pub type BasicConsumer = Consumer<BasicNonce<SystemEpochProvider>>;

#[derive(Debug)]
pub struct Consumer<NP: NonceProvider> {
    consumer_key: String,
    consumer_secret: String,
    nonce_provider: NP,

    request_token_url: Url,
    user_authorization_url: Url,
    access_token_url: Url,

    user_auth_key_param_name: String,
    user_auth_token_param_name: String,

    save_file: Option<PathBuf>,
    state: ConsumerState,
}

impl<NP: NonceProvider> Consumer<NP> {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn is_fully_authed(&self) -> bool {
        matches!(self.state, ConsumerState::FullAuth { .. })
    }

    fn ensure_auth(&mut self) -> Result<()> {
        if !self.is_fully_authed() {
            self.retrieve_request_token()?;

            // TODO: pass in an argument to the consumer on whether to open the browser.
            // TODO: make the messages match what's happening with the browser opening or not.
            let url = self.make_user_auth_url()?;
            println!(
                "Go to the following URL and follow the instructions:\n\n    {}\n\n",
                url
            );
            open::that(url.to_string())?;
            print!("Input the verification code received from the server: ");
            std::io::stdout().flush()?;
            let mut code: String = Default::default();
            std::io::stdin().read_line(&mut code)?;
            self.set_verification_code(code.trim())?;

            self.retrieve_access_token()?;
        }
        Ok(())
    }

    pub fn get(&mut self, url: &Url) -> Result<String> {
        self.ensure_auth()?;

        let auth_header = self.sign_request_from_components("GET", url)?;
        debug!("get: auth_header: {}", auth_header);
        let client = Client::builder().build()?;
        let response = client
            .get(url.clone())
            .header("Authorization", auth_header)
            .send()?;

        // TODO: check for non-200 result code.
        debug!("get: response: {:?}", response);
        let response_str = String::from_utf8(Vec::from(response.bytes()?))?;

        // TODO: add param processing.

        Ok(response_str)
    }

    pub fn get_fake(&mut self, url: &Url) -> Result<String> {
        self.ensure_auth()?;

        let auth_header = self.sign_request_from_components("GET", url)?;
        println!("Method: '{}'", "GET");
        println!("Url:    '{}'", url.to_string());
        println!("Header: '{}'", auth_header);
        Ok("FAKE".to_string())
    }

    pub fn retrieve_request_token(&mut self) -> Result<()> {
        let response = self.canned_request("GET", &self.request_token_url.clone())?;
        let response_str = String::from_utf8(Vec::from(response.bytes()?))?;

        // TODO: check the incoming state.

        let params = decode_params_string(response_str);
        let mut request_token = None;
        let mut token_secret = None;
        for param in params {
            let name = param.name;
            if name == "oauth_token" {
                // TODO: Do you want to check for None?
                request_token = param.value;
            } else if name == "oauth_token_secret" {
                // TODO: Do you want to check for None?
                token_secret = param.value;
            }
        }
        // TODO: At this point, you should check to make sure that both params were present.
        self.state = ConsumerState::RequestToken {
            request_token: request_token.ok_or(OagainError::MissingRequestToken)?,
            token_secret: token_secret.ok_or(OagainError::MissingTokenSecret)?,
        };

        Ok(())
    }

    pub fn set_verification_code(&mut self, code: impl AsRef<str>) -> Result<()> {
        self.state = ConsumerState::UserAuth {
            request_token: self
                .state
                .token()
                .ok_or(OagainError::MissingRequestToken)?
                .to_string(),
            token_secret: self
                .state
                .token_secret()
                .ok_or(OagainError::MissingTokenSecret)?
                .to_string(),
            verification_code: code.as_ref().to_string(),
        };
        Ok(())
    }

    fn toml_now() -> toml::value::Datetime {
        let utc = Utc::now();
        toml::value::Datetime {
            date: Some(toml::value::Date {
                year: utc.year() as u16,
                month: utc.month() as u8,
                day: utc.day() as u8,
            }),
            time: Some(toml::value::Time {
                hour: utc.hour() as u8,
                minute: utc.minute() as u8,
                second: utc.second() as u8,
                nanosecond: 0,
            }),
            offset: Some(toml::value::Offset::Z),
        }
    }

    fn write_state_to_save_file(&self) -> Result<()> {
        if let Some(save_file) = &self.save_file {
            let mut table = toml::Table::new();
            table.insert(
                ACCESS_TOKEN_NAME.to_string(),
                Value::String(self.state.token().unwrap_or_default().to_string()),
            );
            table.insert(
                TOKEN_SECRET_NAME.to_string(),
                Value::String(self.state.token_secret().unwrap_or_default().to_string()),
            );
            table.insert(
                TOKEN_SAVE_TIME.to_string(),
                Value::Datetime(Self::toml_now()),
            );

            let mut f = File::create(save_file)?;
            f.write_all(table.to_string().as_bytes())?;
        }
        Ok(())
    }

    pub fn retrieve_access_token(&mut self) -> Result<()> {
        debug!("retrieve_access_token: {:?}", self);
        let response = self.canned_request("GET", &self.access_token_url.clone())?;
        debug!("access raw response: {:?}", response);
        let response_str: String = String::from_utf8(Vec::from(response.bytes()?))?;
        debug!("access response: {}", response_str);

        let params = decode_params_string(response_str);
        let mut access_token = None;
        let mut token_secret = None;
        for param in params {
            if param.name == OAUTH_TOKEN_PARAM_NAME {
                access_token = param.value;
            } else if param.name == OAUTH_TOKEN_SECRET_PARAM_NAME {
                token_secret = param.value;
            }
        }

        self.state = ConsumerState::FullAuth {
            access_token: access_token.ok_or(OagainError::MissingAccessToken)?,
            token_secret: token_secret.ok_or(OagainError::MissingTokenSecret)?,
        };

        if let Err(err) = self.write_state_to_save_file() {
            error!("Failed writing to save file: {}", err.to_string())
        }

        Ok(())
    }

    pub fn make_user_auth_url(&mut self) -> Result<Url> {
        let mut base_url = self.user_authorization_url.clone();
        let request_token = self.state.token().ok_or(OagainError::MissingRequestToken)?;
        base_url
            .query_pairs_mut()
            .append_pair(&self.user_auth_token_param_name, request_token)
            .append_pair(&self.user_auth_key_param_name, &self.consumer_key);
        Ok(base_url)
    }

    fn canned_request(&mut self, method: impl AsRef<str>, url: &Url) -> Result<Response> {
        //let url = url(self).clone();
        let auth_header = self.sign_request_from_components(method, url)?;
        debug!("auth_header: {}", auth_header);
        // TODO: reuse these clients.
        let client = Client::builder().build()?;
        // TODO: make this a fold()
        let response = client
            .get(url.clone())
            .header("Authorization", auth_header)
            .send()?;

        Ok(response)
    }

    fn sign_request_from_components(
        &mut self,
        method: impl AsRef<str>,
        url: &Url,
    ) -> Result<String> {
        let (timestamp, nonce) = self.nonce()?;
        debug!("timestamp, nonce: {}, {}", timestamp, nonce);
        let standard_params = self.oauth_param_list(timestamp, nonce);
        debug!("standard_params: {:?}", standard_params);

        let other_params = url
            .query_pairs()
            .map(|(name, value)| ParamPair::pair(name, value))
            .collect::<Vec<_>>();

        let param_iter = standard_params.iter().chain(other_params.iter());

        let string_to_sign = concat_request_elements(method.as_ref(), url, param_iter.cloned());
        debug!("string_to_sign: {}", string_to_sign);

        let signing_key = make_signing_key(
            &self.consumer_secret,
            self.state.token_secret().unwrap_or_default(),
        );
        debug!("signing_key: {}", signing_key);
        let signature = sign_string_hmac(signing_key, string_to_sign);

        let header = self.oauth_header(&standard_params, signature);
        Ok(header)
    }

    pub(crate) fn oauth_param_list(
        &self,
        timestamp: u32,
        nonce: impl AsRef<str>,
    ) -> Vec<ParamPair> {
        let (include_callback, include_token, include_verifier) = match &self.state {
            ConsumerState::NoAuth => (true, false, false),
            ConsumerState::RequestToken { .. } => (false, true, false),
            ConsumerState::UserAuth { .. } => (false, true, true),
            ConsumerState::FullAuth { .. } => (false, true, false),
        };
        let pair_descriptions: &[(&'static str, &dyn Fn() -> Option<String>)] = &[
            (OAUTH_CONSUMER_KEY_PARAM_NAME, &|| {
                Some(self.consumer_key.clone())
            }),
            (OAUTH_SIGNATURE_METHOD_PARAM_NAME, &|| {
                Some(OAUTH_SIGNATURE_METHOD_HMAC_VALUE.to_string())
            }),
            (OAUTH_TIMESTAMP_PARAM_NAME, &|| Some(timestamp.to_string())),
            (OAUTH_NONCE_PARAM_NAME, &|| {
                nonce.as_ref().to_string().into()
            }),
            (OAUTH_VERSION_PARAM_NAME, &|| {
                OAUTH_VERSION_VALUE.to_string().into()
            }),
            (OAUTH_CALLBACK_PARAM_NAME, &|| {
                include_callback.option(OAUTH_CALLBACK_OOB_VALUE.to_string())
            }),
            (OAUTH_TOKEN_PARAM_NAME, &|| {
                // TODO: bad unwrap
                // TODO: need to use access_token sometimes.
                include_token.option_with(|| self.state.token().unwrap().to_string())
            }),
            (OAUTH_VERIFIER_PARAM_NAME, &|| {
                // TODO: bad unwrap
                include_verifier.option_with(|| self.state.verification_code().unwrap().to_string())
            }),
        ];

        pair_descriptions
            .iter()
            .fold(Vec::default(), |mut acc, (name, value_func)| {
                if let Some(val) = value_func() {
                    acc.push(ParamPair::pair(*name, val));
                }
                acc
            })
    }

    //----------------------------------------------------------------------

    pub(crate) fn nonce(&mut self) -> Result<(u32, String)> {
        self.nonce_provider.nonce()
    }

    pub(crate) fn oauth_header(
        &self,
        param_pairs: &[ParamPair],
        signature: impl AsRef<str>,
    ) -> String {
        let signature_pair = ParamPair::pair(OAUTH_SIGNATURE_PARAM_NAME, signature.as_ref());
        format!(
            "OAuth {}",
            param_pairs
                .iter()
                .sorted()
                .chain(once(&signature_pair))
                .map(|pp| pp.to_wrapped_string())
                .join(", ")
        )
    }
}

#[cfg(test)]
pub(crate) trait ConsumerTestFuncs {
    fn request_url(&self) -> &Url;
}

#[cfg(test)]
impl<NP: NonceProvider> ConsumerTestFuncs for Consumer<NP> {
    fn request_url(&self) -> &Url {
        &self.request_token_url
    }
}
