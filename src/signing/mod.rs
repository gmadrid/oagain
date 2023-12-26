#[cfg(test)]
mod test_cases;

mod signature_base_string;
mod signer;

use crate::pencoding::encode_param;
use base64::Engine;
use crypto::mac::Mac;
pub use signature_base_string::concat_request_elements;

pub fn make_signing_key(consumer_secret: impl AsRef<str>, token: impl AsRef<str>) -> String {
    format!("{}&{}", encode_param(consumer_secret), encode_param(token))
}

pub fn sign_string_hmac(key: impl AsRef<str>, text: impl AsRef<str>) -> String {
    use crypto::{hmac, sha1};
    let mut hmac = hmac::Hmac::new(sha1::Sha1::new(), key.as_ref().as_bytes());
    hmac.input(text.as_ref().as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hmac.result().code())
}
