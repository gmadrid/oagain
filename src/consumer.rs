use itertools::Itertools;
use reqwest::blocking::Client;
use std::iter::once;
use url::Url;

use crate::config::Config;
use crate::constants::{
    OAUTH_CALLBACK_OOB_VALUE, OAUTH_CALLBACK_PARAM_NAME, OAUTH_CONSUMER_KEY_PARAM_NAME,
    OAUTH_NONCE_PARAM_NAME, OAUTH_SIGNATURE_METHOD_HMAC_VALUE, OAUTH_SIGNATURE_METHOD_PARAM_NAME,
    OAUTH_SIGNATURE_PARAM_NAME, OAUTH_TIMESTAMP_PARAM_NAME, OAUTH_VERSION_PARAM_NAME,
    OAUTH_VERSION_VALUE,
};
use crate::nonce_provider::{BasicNonce, NonceProvider, SystemEpochProvider};
use crate::parameters::ParamPair;
use crate::signing::{concat_request_elements, make_signing_key, sign_string_hmac};
use crate::Result;

/// A basic consumer that uses the standard time-based nonce provider.
pub type BasicConsumer = Consumer<BasicNonce<SystemEpochProvider>>;

#[derive(Debug)]
pub struct Consumer<NP: NonceProvider> {
    consumer_key: String,
    consumer_secret: String,
    nonce_provider: NP,
    config: Config,

    request_token: Option<Vec<u8>>,
    access_token: Option<Vec<u8>>,
    token_secret: Option<Vec<u8>>,
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
            request_token: None,
            access_token: None,
            token_secret: None,
        })
    }

    //----------------------------------------------------------------------

    pub fn obtain_request_token(&mut self) -> Result<()> {
        let (timestamp, nonce) = self.nonce()?;
        let mut params = self.oauth_standard_param_pairs(timestamp, &nonce);

        // the token is "" because there is no token for this request.
        let signing_key = self.request_token_signing_key()?;
        let string_to_sign =
            concat_request_elements("POST", &self.config.request_token_url, &params);
        let signature = sign_string_hmac(signing_key, string_to_sign);

        // callback is not a standard param.
        params.push(ParamPair::pair(
            OAUTH_CALLBACK_PARAM_NAME,
            OAUTH_CALLBACK_OOB_VALUE,
        ));
        params.push(ParamPair::pair(OAUTH_SIGNATURE_PARAM_NAME, signature));

        // Make the payload string.
        let auth_header_params = params.iter().map(|pp| pp.to_wrapped_string()).join(",");
        let auth_header_value = format!("OAuth {}", auth_header_params);
        println!("Authorization: {}", auth_header_value);

        // TODO: set some default headers
        let client = Client::builder().build()?;
        let request = client
            .post(self.config.request_token_url.clone())
            .header("Authorization", auth_header_value);

        println!("\nRequest: {:?}", request);
        let response = request.send()?;

        println!("\nResponse: {:?}", response);

        Ok(())
    }

    pub fn request_token_signing_key(&self) -> Result<String> {
        Ok(make_signing_key(&self.consumer_secret, ""))
    }

    pub fn nonce(&mut self) -> Result<(u32, String)> {
        self.nonce_provider.nonce()
    }

    pub fn oauth_standard_param_pairs(&mut self, timestamp: u32, nonce: &str) -> Vec<ParamPair> {
        let mut params = vec![
            ParamPair::pair(OAUTH_CONSUMER_KEY_PARAM_NAME, &self.consumer_key),
            ParamPair::pair(
                OAUTH_SIGNATURE_METHOD_PARAM_NAME,
                OAUTH_SIGNATURE_METHOD_HMAC_VALUE,
            ),
            ParamPair::pair(OAUTH_TIMESTAMP_PARAM_NAME, timestamp.to_string()),
            ParamPair::pair(OAUTH_NONCE_PARAM_NAME, nonce),
            ParamPair::pair(OAUTH_VERSION_PARAM_NAME, OAUTH_VERSION_VALUE),
        ];
        params
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
