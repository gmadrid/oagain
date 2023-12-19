use crate::constants::{OAUTH_CALLBACK_OOB_VALUE, OAUTH_CALLBACK_PARAM_NAME};
use crate::consumer::Consumer;
use crate::consumer::ConsumerTestFuncs;
use crate::nonce_provider::{BasicNonce, TestEpochProvider};
use crate::parameters::ParamPair;
use crate::signing::{concat_request_elements, sign_string_hmac};
use crate::test_constants::{ACCESS_TOKEN_URL, REQUEST_TOKEN_URL, USER_AUTHORIZATION_URL};
use crate::Config;

/// https://lti.tools/oauth/ has tools for generating signatures (and all of the intermediates)
/// for a user-supplied configuration.
///
/// This file uses this site and my _old_ eTrade keys to generate correct values for
/// test cases that mirror the data for eTrade.

fn config_with_etrade_urls() -> Config {
    Config {
        request_token_url: REQUEST_TOKEN_URL.parse().unwrap(),
        user_authorization_url: USER_AUTHORIZATION_URL.parse().unwrap(),
        access_token_url: ACCESS_TOKEN_URL.parse().unwrap(),
    }
}

fn consumer_with_known_timestamp(timestamp: u32) -> Consumer<BasicNonce<TestEpochProvider>> {
    let epoch_provider = TestEpochProvider::new(timestamp);
    let nonce_provider = BasicNonce::new(epoch_provider);

    Consumer::new_with_nonce(
        "f94997add0b18f6c81e43b9843149042",
        "56d240d097f004525b6a1ed6fba27343",
        config_with_etrade_urls(),
        nonce_provider,
    )
    .unwrap()
}

// This is December 18, 2023, 12:18:23UTC
const TEST_TIMESTAMP: u32 = 1702901903;

#[test]
fn test_request_token() {
    let mut consumer = consumer_with_known_timestamp(TEST_TIMESTAMP);
    let method = "GET";
    let url = consumer.request_url().clone();

    //consumer.request_url().clone();

    let (timestamp, nonce) = consumer.nonce().unwrap();
    let pairs = consumer.oauth_standard_param_pairs(timestamp, &nonce);
    println!("timestamp: {}, nonce: {}", timestamp, nonce);

    let mut all_pairs = pairs.clone();
    all_pairs.push(ParamPair::pair(
        OAUTH_CALLBACK_PARAM_NAME,
        OAUTH_CALLBACK_OOB_VALUE,
    ));

    let string_to_sign = concat_request_elements(method, &url, &all_pairs);

    let signature_base_string: &str =
    "GET&https%3A%2F%2Fphotos.example.net%2Frequest_token&oauth_callback%3Doob%26oauth_consumer_key%3Df94997add0b18f6c81e43b9843149042%26oauth_nonce%3Dnonce-1702901903-0%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1702901903%26oauth_version%3D1.0";
    assert_eq!(signature_base_string, string_to_sign);

    let signing_key = consumer.request_token_signing_key().unwrap();
    assert_eq!("56d240d097f004525b6a1ed6fba27343&", signing_key);

    let signature = sign_string_hmac(signing_key, signature_base_string);
    assert_eq!("6cTuGtNttPj1MotXdq2QYesjJ6g=", signature);

    let expected_header = r#"OAuth oauth_consumer_key="f94997add0b18f6c81e43b9843149042", oauth_nonce="nonce-1702901903-0", oauth_signature_method="HMAC-SHA1", oauth_timestamp="1702901903", oauth_version="1.0", oauth_signature="6cTuGtNttPj1MotXdq2QYesjJ6g%3D""#;
    let oauth_header = consumer.oauth_header(&pairs, signature);
    assert_eq!(expected_header, oauth_header);
}
