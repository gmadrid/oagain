use super::RequestScheme;
use crate::consumer::Consumer;
use crate::nonce_provider::NonceProvider;
use crate::parameters::ParamPair;
use crate::OagainError;
use url::Url;

pub struct AccessTokenScheme;

impl RequestScheme for AccessTokenScheme {
    fn name(&self) -> &'static str {
        "AccessTokenScheme"
    }

    fn extra_params(&self) -> Vec<ParamPair> {
        vec![]
    }

    fn token<'a, NP: NonceProvider>(
        &self,
        consumer: &'a Consumer<NP>,
    ) -> crate::error::Result<&'a str> {
        Ok(consumer
            .state
            .token()
            .ok_or(OagainError::MissingRequestToken)?)
    }

    fn method(&self) -> &'static str {
        "GET"
    }

    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url {
        &consumer.access_token_url
    }
}
