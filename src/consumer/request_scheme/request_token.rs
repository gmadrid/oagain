use super::RequestScheme;
use crate::consumer::Consumer;
use crate::nonce_provider::NonceProvider;
use crate::parameters::ParamPair;
use url::Url;

pub struct RequestTokenScheme;

impl RequestScheme for RequestTokenScheme {
    // TODO: return an Option
    fn extra_params(&self) -> Vec<ParamPair> {
        vec![]
    }

    fn token<'a, NP: NonceProvider>(&self, _consumer: &'a Consumer<NP>) -> crate::Result<&'a str> {
        Ok("")
    }

    fn method(&self) -> &'static str {
        "GET"
    }

    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url {
        &consumer.config.request_token_url
    }
}
