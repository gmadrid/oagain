mod access_token;
mod request_token;

use crate::consumer::Consumer;
use crate::nonce_provider::NonceProvider;
use crate::parameters::ParamPair;
use url::Url;

pub use access_token::AccessTokenScheme;
pub use request_token::RequestTokenScheme;

/*
   - All request headers
     - oauth_consumer_key
     - oauth_signature_method
     - oauth_timestamp
     - oauth_nonce
     - oauth_version
   - Other request headers
     - oauth_callback
     - oauth_signature
     - oauth_token
     - oauth_verifier

   - Request token headers
     - oauth_signature
     - oauth_callback
   - Access token headers
     - oauth_token
     - oauth_signature
     - oauth_verifier
   - Other request headers
     - oauth_token
     - oauth_signature

*/

pub trait RequestScheme {
    fn extra_params(&self) -> Vec<ParamPair>;
    fn token<'a, NP: NonceProvider>(
        &self,
        consumer: &'a Consumer<NP>,
    ) -> crate::error::Result<&'a str>;
    fn method(&self) -> &'static str;
    fn url<'a, NP: NonceProvider>(&self, consumer: &'a Consumer<NP>) -> &'a Url;
}
