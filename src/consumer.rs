use itertools::Itertools;
use reqwest::blocking::Client;

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
    pub fn new(
        key: impl Into<String>,
        secret: impl Into<String>,
        config: Config,
    ) -> Result<BasicConsumer> {
        Consumer::new_with_nonce(key, secret, config, BasicNonce::default())
    }

    pub fn new_with_nonce(
        key: impl Into<String>,
        secret: impl Into<String>,
        config: Config,
        nonce: NP,
    ) -> Result<Self> {
        Ok(Consumer {
            consumer_key: key.into(),
            consumer_secret: secret.into(),
            nonce_provider: nonce,
            config,
            request_token: None,
            access_token: None,
            token_secret: None,
        })
    }

    pub fn obtain_request_token(&mut self) -> Result<()> {
        let (timestamp, nonce) = self.nonce_provider.nonce()?;
        let mut params = vec![
            ParamPair::pair(OAUTH_CONSUMER_KEY_PARAM_NAME, Some(&self.consumer_key)),
            ParamPair::pair(
                OAUTH_SIGNATURE_METHOD_PARAM_NAME,
                Some(OAUTH_SIGNATURE_METHOD_HMAC_VALUE),
            ),
            ParamPair::pair(OAUTH_TIMESTAMP_PARAM_NAME, Some(timestamp.to_string())),
            ParamPair::pair(OAUTH_NONCE_PARAM_NAME, Some(nonce)),
            ParamPair::pair(OAUTH_VERSION_PARAM_NAME, Some(OAUTH_VERSION_VALUE)),
            ParamPair::pair(OAUTH_CALLBACK_PARAM_NAME, Some(OAUTH_CALLBACK_OOB_VALUE)),
        ];

        // the token is "" because there is no token for this request.
        let signing_key = make_signing_key(&self.consumer_secret, "");
        let string_to_sign =
            concat_request_elements("POST", &self.config.request_token_url, &params);
        let signature = sign_string_hmac(signing_key, string_to_sign);

        params.push(ParamPair::pair(OAUTH_SIGNATURE_PARAM_NAME, Some(signature)));

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
}

#[cfg(test)]
mod test {
    use crate::config::Config;
    use crate::consumer::BasicConsumer;
    use crate::test_constants::{ACCESS_TOKEN_URL, REQUEST_TOKEN_URL, USER_AUTHORIZATION_URL};
    use url::Url;

    #[test]
    fn foobasr() {
        let mut consumer = BasicConsumer::new(
            "dpf43f3p2l4k3l03",
            "kd94hf93k423kf44",
            Config {
                request_token_url: Url::parse(REQUEST_TOKEN_URL).unwrap(),
                user_authorization_url: Url::parse(USER_AUTHORIZATION_URL).unwrap(),
                access_token_url: Url::parse(ACCESS_TOKEN_URL).unwrap(),
            },
        )
        .unwrap();
        consumer.obtain_request_token().unwrap();
        assert!(false)
    }
}

/*
    An example from eTrade:

     https://api.etrade.com/oauth/request_token

     Authorization: OAuth realm="",oauth_callback="oob",
oauth_signature="FjoSQaFDKEDK1FJazlY3xArNflk%3D", oauth_nonce="LTg2ODUzOTQ5MTEzMTY3MzQwMzE%3D",
oauth_signature_method="HMAC-SHA1",oauth_consumer_key="282683cc9e4b8fc81dea6bc687d46758",
oauth_timestamp="1273254425"

     Response:
     oauth_token=%2FiQRgQCRGPo7Xdk6G8QDSEzX0Jsy6sKNcULcDavAGgU%3D&amp;oauth_token_secret=%2FrC9scEpzcwSEMy4vE7nodSzPLqfRINnTNY4voczyFM%3D&amp;oauth_callback_confirmed=true

     NOTE: I'm not sure that the signature is real. It's the same for all of the eTrade examples.
*/
