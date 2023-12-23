use crate::consumer::request_scheme::{RequestScheme, RequestTokenScheme};
use crate::consumer::Consumer;
use crate::consumer::ConsumerTestFuncs;
use crate::nonce_provider::{BasicNonce, TestEpochProvider};
use crate::signing::{concat_request_elements, make_signing_key, sign_string_hmac};
use crate::test_constants::{ACCESS_TOKEN_URL, REQUEST_TOKEN_URL, USER_AUTHORIZATION_URL};
use crate::Config;
use url::Url;

/// https://lti.tools/oauth/ has tools for generating signatures (and all of the intermediates)
/// for a user-supplied configuration.
///
/// This file uses this site and my _old_ eTrade keys to generate correct values for
/// test cases that mirror the data for eTrade.

fn config_with_etrade_urls() -> Config {
    Config::new(REQUEST_TOKEN_URL, USER_AUTHORIZATION_URL, ACCESS_TOKEN_URL).unwrap()
}

fn consumer_with_known_timestamp(timestamp: u32) -> Consumer<BasicNonce<TestEpochProvider>> {
    let epoch_provider = TestEpochProvider::new(timestamp);
    let nonce_provider = BasicNonce::new(epoch_provider);

    Consumer::<BasicNonce<TestEpochProvider>>::builder()
        .set_request_token_url(Url::parse(REQUEST_TOKEN_URL).unwrap())
        .set_user_authorization_url(Url::parse(USER_AUTHORIZATION_URL).unwrap())
        .set_access_token_url(Url::parse(ACCESS_TOKEN_URL).unwrap())
        .set_consumer_key("f94997add0b18f6c81e43b9843149042")
        .set_consumer_secret("56d240d097f004525b6a1ed6fba27343")
        .build_with_nonce_provider(nonce_provider)
        .unwrap()
}

// This is December 18, 2023, 12:18:23UTC
const TEST_TIMESTAMP: u32 = 1702901903;

#[test]
fn test_request_token() {
    let mut consumer = consumer_with_known_timestamp(TEST_TIMESTAMP);
    let method = "GET";
    let url = consumer.request_url().clone();

    let (timestamp, nonce) = consumer.nonce().unwrap();
    let pairs = consumer.oauth_param_list(timestamp, &nonce, true, false, false);

    let string_to_sign = concat_request_elements(method, &url, pairs.iter().cloned());

    let signature_base_string: &str =
    "GET&https%3A%2F%2Fphotos.example.net%2Frequest_token&oauth_callback%3Doob%26oauth_consumer_key%3Df94997add0b18f6c81e43b9843149042%26oauth_nonce%3Dnonce-1702901903-0%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1702901903%26oauth_version%3D1.0";
    assert_eq!(signature_base_string, string_to_sign);

    let signing_key = make_signing_key(
        &consumer.consumer_secret,
        RequestTokenScheme.token(&consumer).unwrap(),
    );
    //let signing_key = consumer.request_token_signing_key().unwrap();
    assert_eq!("56d240d097f004525b6a1ed6fba27343&", signing_key);

    let signature = sign_string_hmac(signing_key, signature_base_string);
    assert_eq!("6cTuGtNttPj1MotXdq2QYesjJ6g=", signature);

    let expected_header = r#"OAuth oauth_callback="oob", oauth_consumer_key="f94997add0b18f6c81e43b9843149042", oauth_nonce="nonce-1702901903-0", oauth_signature_method="HMAC-SHA1", oauth_timestamp="1702901903", oauth_version="1.0", oauth_signature="6cTuGtNttPj1MotXdq2QYesjJ6g%3D""#;
    let oauth_header = consumer.oauth_header(&pairs, signature);
    assert_eq!(expected_header, oauth_header);
}
