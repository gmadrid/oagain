use url::Url;

use crate::consumer::Consumer;
use crate::nonce_provider::NonceProvider;

use super::RequestScheme;

pub struct RequestTokenScheme;

impl RequestScheme for RequestTokenScheme {
    // TODO: return an Option
    fn name(&self) -> &'static str {
        "RequestTokenScheme"
    }

    fn method(&self) -> &'static str {
        "GET"
    }

    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url {
        &consumer.request_token_url
    }
}
