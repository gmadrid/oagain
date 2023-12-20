use super::RequestScheme;
use crate::consumer::Consumer;
use crate::nonce_provider::NonceProvider;
use crate::parameters::ParamPair;
use url::Url;

pub struct AccessTokenScheme;

impl RequestScheme for AccessTokenScheme {
    fn extra_params(&self) -> Vec<ParamPair> {
        todo!()
    }

    fn token<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> crate::Result<&'a str> {
        todo!()
    }

    fn method(&self) -> &'static str {
        todo!()
    }

    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url {
        todo!()
    }
}
