use std::iter::once;

use itertools::Itertools;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use url::Url;

use request_scheme::AccessTokenScheme;
use request_scheme::RequestScheme;
use request_scheme::RequestTokenScheme;

use crate::constants::{
    OAUTH_CALLBACK_OOB_VALUE, OAUTH_CALLBACK_PARAM_NAME, OAUTH_CONSUMER_KEY_PARAM_NAME,
    OAUTH_NONCE_PARAM_NAME, OAUTH_SIGNATURE_METHOD_HMAC_VALUE, OAUTH_SIGNATURE_METHOD_PARAM_NAME,
    OAUTH_SIGNATURE_PARAM_NAME, OAUTH_TIMESTAMP_PARAM_NAME, OAUTH_TOKEN_PARAM_NAME,
    OAUTH_VERIFIER_PARAM_NAME, OAUTH_VERSION_PARAM_NAME, OAUTH_VERSION_VALUE,
};
use crate::consumer::builder::Builder;
use crate::error::{OagainError, Result};
use crate::nonce_provider::{BasicNonce, NonceProvider, SystemEpochProvider};
use crate::parameters::{decode_params_string, ParamPair};
use crate::signing::{concat_request_elements, make_signing_key, sign_string_hmac};
use crate::util::BoolToOption;
pub use builder::preset::ETradePreset;
use state::ConsumerState;

mod builder;
pub(crate) mod request_scheme;
mod state;

/// A basic consumer that uses the standard time-based nonce provider.
pub type BasicConsumer = Consumer<BasicNonce<SystemEpochProvider>>;

#[derive(Debug)]
pub struct Consumer<NP: NonceProvider> {
    pub(crate) consumer_key: String,
    pub(crate) consumer_secret: String,
    nonce_provider: NP,

    request_token_url: Url,
    user_authorization_url: Url,
    access_token_url: Url,

    user_auth_key_param_name: String,
    user_auth_token_param_name: String,

    state: ConsumerState,
}

impl<NP: NonceProvider> Consumer<NP> {
    pub fn builder() -> Builder {
        Builder::default()
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
        let mut base_url = self.user_authorization_url.clone();
        let request_token = self.state.token().ok_or(OagainError::MissingRequestToken)?;
        base_url
            .query_pairs_mut()
            .append_pair(&self.user_auth_token_param_name, request_token)
            .append_pair(&self.user_auth_key_param_name, &self.consumer_key);
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
        &self.request_token_url
    }
}
