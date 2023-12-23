use std::iter::once;

use itertools::Itertools;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use url::Url;

use request_scheme::AccessTokenScheme;
use request_scheme::RequestScheme;
use request_scheme::RequestTokenScheme;

use crate::config::Config;
use crate::constants::{
    OAUTH_CALLBACK_OOB_VALUE, OAUTH_CALLBACK_PARAM_NAME, OAUTH_CONSUMER_KEY_PARAM_NAME,
    OAUTH_NONCE_PARAM_NAME, OAUTH_SIGNATURE_METHOD_HMAC_VALUE, OAUTH_SIGNATURE_METHOD_PARAM_NAME,
    OAUTH_SIGNATURE_PARAM_NAME, OAUTH_TIMESTAMP_PARAM_NAME, OAUTH_TOKEN_PARAM_NAME,
    OAUTH_VERIFIER_PARAM_NAME, OAUTH_VERSION_PARAM_NAME, OAUTH_VERSION_VALUE,
};
use crate::nonce_provider::{BasicNonce, NonceProvider, SystemEpochProvider};
use crate::parameters::{decode_params_string, ParamPair};
use crate::signing::{concat_request_elements, make_signing_key, sign_string_hmac};
use crate::{OagainError, Result};

pub(crate) mod request_scheme;

trait BoolToOption<T> {
    fn option(self, val: T) -> Option<T>;
    fn option_with(self, val_func: impl Fn() -> T) -> Option<T>;
}

impl<T> BoolToOption<T> for bool {
    fn option(self, val: T) -> Option<T> {
        if self {
            val.into()
        } else {
            None
        }
    }

    fn option_with(self, val_func: impl Fn() -> T) -> Option<T> {
        if self {
            val_func().into()
        } else {
            None
        }
    }
}

/// A basic consumer that uses the standard time-based nonce provider.
pub type BasicConsumer = Consumer<BasicNonce<SystemEpochProvider>>;

#[derive(Debug)]
pub struct Consumer<NP: NonceProvider> {
    pub(crate) consumer_key: String,
    pub(crate) consumer_secret: String,
    nonce_provider: NP,
    config: Config,

    state: ConsumerState,
}

#[derive(Debug, Default)]
pub enum ConsumerState {
    #[default]
    NoAuth,

    RequestToken {
        request_token: String,
        token_secret: String,
    },

    UserAuth {
        request_token: String,
        token_secret: String,
        verification_code: String,
    },

    FullAuth {
        access_token: String,
        token_secret: String,
    },
}

impl ConsumerState {
    fn request_token(&self) -> Option<&str> {
        match self {
            ConsumerState::RequestToken { request_token, .. } => Some(request_token),
            ConsumerState::UserAuth { request_token, .. } => Some(request_token),
            _ => None,
        }
    }

    fn token_secret(&self) -> Option<&str> {
        match self {
            ConsumerState::RequestToken { token_secret, .. } => Some(token_secret),
            ConsumerState::UserAuth { token_secret, .. } => Some(token_secret),
            ConsumerState::FullAuth { token_secret, .. } => Some(token_secret),
            ConsumerState::NoAuth => None,
        }
    }

    fn verification_code(&self) -> Option<&str> {
        match self {
            ConsumerState::UserAuth {
                verification_code, ..
            } => Some(verification_code),
            _ => None,
        }
    }

    fn access_token(&self) -> Option<&str> {
        match self {
            ConsumerState::FullAuth { access_token, .. } => Some(access_token),
            _ => None,
        }
    }
}

impl<NP: NonceProvider> Consumer<NP> {
    /// Creates a new BasicConsumer with the provided `consumer_key`, `consumer_secret`, and `config`.
    pub fn new(
        consumer_key: impl Into<String>,
        consumer_secret: impl Into<String>,
        config: Config,
    ) -> Result<BasicConsumer> {
        Consumer::new_with_nonce(consumer_key, consumer_secret, config, BasicNonce::default())
    }

    /// Creates a new Consumer with the provided `consumer_key`, `consumer_secret`, and `config`.
    /// The new consumer will used the provided `nonce_provider` for generating nonces.
    pub fn new_with_nonce(
        consumer_key: impl Into<String>,
        consumer_secret: impl Into<String>,
        config: Config,
        nonce_provider: NP,
    ) -> Result<Self> {
        Ok(Consumer {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            nonce_provider,
            config,
            state: Default::default(),
        })
    }

    pub fn retrieve_request_token(&mut self) -> Result<()> {
        let response = self.canned_request(&RequestTokenScheme)?;
        let response_str: String = String::from_utf8(Vec::from(response.bytes()?))?;

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

    pub fn retrieve_access_token(&mut self) -> Result<()> {
        let response = self.canned_request(&AccessTokenScheme)?;
        let response_str: String = String::from_utf8(Vec::from(response.bytes()?))?;
        Ok(())
    }

    pub fn make_user_auth_url(&mut self) -> Result<Url> {
        let mut base_url = self.config.user_authorization_url.clone();
        let request_token = self
            .state
            .request_token()
            .ok_or(OagainError::MissingRequestToken)?;
        base_url
            .query_pairs_mut()
            .append_pair(&self.config.user_auth_token_param_name, request_token)
            .append_pair(&self.config.user_auth_key_param_name, &self.consumer_key);
        Ok(base_url)
    }

    fn canned_request(&mut self, req: &impl RequestScheme) -> Result<Response> {
        let auth_header = self.sign_request_from_components(req)?;
        // TODO: reuse these clients.
        let client = Client::builder().build()?;
        // TODO: make this a fold()
        let mut url = req.url(self).clone();
        {
            let mut pairs = url.query_pairs_mut();
            for param in req.extra_params() {
                pairs.append_pair(&param.name, &param.value.unwrap_or_default());
            }
        }
        let response = client
            .get(url)
            .header("Authorization", auth_header)
            .send()?;

        Ok(response)
    }

    fn sign_request_from_components(&mut self, req: &impl RequestScheme) -> Result<String> {
        let (timestamp, nonce) = self.nonce()?;
        //let standard_params = self.oauth_standard_param_pairs(timestamp, &nonce, true);
        let standard_params = self.oauth_param_list(timestamp, nonce, true, false, false);
        let extra_params = req.extra_params();
        let string_to_sign = {
            let all_params = standard_params.iter().cloned().chain(extra_params);
            concat_request_elements(req.method(), req.url(self), all_params)
        };

        let signing_key = make_signing_key(&self.consumer_secret, req.token(self)?);
        let signature = sign_string_hmac(signing_key, string_to_sign);

        let header = self.oauth_header(&standard_params, signature);
        Ok(header)
    }

    pub(crate) fn oauth_param_list(
        &self,
        timestamp: u32,
        nonce: impl AsRef<str>,
        include_callback: bool,
        include_token: bool,
        include_verifier: bool,
    ) -> Vec<ParamPair> {
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
                include_token.option_with(|| self.state.request_token().unwrap().to_string())
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

    pub fn nonce(&mut self) -> Result<(u32, String)> {
        self.nonce_provider.nonce()
    }

    pub fn oauth_header(&self, param_pairs: &[ParamPair], signature: impl AsRef<str>) -> String {
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
        &self.config.request_token_url
    }
}
