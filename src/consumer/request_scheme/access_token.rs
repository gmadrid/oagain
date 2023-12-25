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

    fn method(&self) -> &'static str {
        "GET"
    }

    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url {
        &consumer.access_token_url
    }
}
