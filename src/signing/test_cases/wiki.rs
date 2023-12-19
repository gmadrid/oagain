use crate::parameters::ParamPair;
use crate::pencoding::encode_param;
use crate::signing::signature_base_string::normalize_request_parameters;
use crate::signing::{make_signing_key, sign_string_hmac};
use std::iter::once;

/// https://wiki.oauth.net/w/page/12238556/TestCases includes many test_cases cases, many derived
/// directly from the OAuth Spec.
///
/// This module runs my code against these tests that have __known__ values, in particular,
/// known signatures. (Notably, the spec includes _no_ signatures.)

#[test]
fn parameter_encoding_sec_5_1() {
    assert_eq!("abcABC123", encode_param("abcABC123"));
    assert_eq!("-._~", encode_param("-._~"));
    assert_eq!("%25", encode_param("%"));
    assert_eq!("%2B", encode_param("+"));
    assert_eq!("%26%3D%2A", encode_param("&=*"));
    assert_eq!("%0A", encode_param("\u{0A}"));
    assert_eq!("%20", encode_param(" "));
    assert_eq!("%7F", encode_param("\u{7F}"));
    assert_eq!("%C2%80", encode_param("\u{80}"));
    assert_eq!("%E3%80%81", encode_param("\u{3001}"));
}

#[test]
fn normalize_request_parameters_sec_9_1_1() {
    assert_eq!(
        "name=",
        normalize_request_parameters(once(ParamPair::single("name")))
    );
    assert_eq!(
        "a=b",
        normalize_request_parameters(once(ParamPair::pair("a", "b")))
    );
    assert_eq!(
        "a=b&c=d",
        normalize_request_parameters(
            [ParamPair::pair("a", "b"), ParamPair::pair("c", "d")].into_iter()
        )
    );
    assert_eq!(
        "a=x%20y&a=x%21y",
        normalize_request_parameters(
            [ParamPair::pair("a", "x!y"), ParamPair::pair("a", "x y")].into_iter()
        )
    );
    assert_eq!(
        "x=a&x%21y=a",
        normalize_request_parameters(
            [ParamPair::pair("x!y", "a"), ParamPair::pair("x", "a")].into_iter()
        )
    );
}

#[test]
fn hmac_sha1_sec_9_2() {
    assert_eq!(
        "egQqG5AJep5sJ7anhXju1unge2I=",
        sign_string_hmac(make_signing_key("cs", ""), "bs")
    );
    assert_eq!(
        "VZVjXceV7JgPq/dOTnNmEfO0Fv8=",
        sign_string_hmac(make_signing_key("cs", "ts"), "bs")
    );

    let long_base_string = "GET&http%3A%2F%2Fphotos.example.net%2Fphotos&file%3Dvacation.jpg%26oauth_consumer_key%3Ddpf43f3p2l4k3l03%26oauth_nonce%3Dkllo9940pd9333jh%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1191242096%26oauth_token%3Dnnch734d00sl2jdk%26oauth_version%3D1.0%26size%3Doriginal";
    assert_eq!(
        "tR3+Ty81lMeYAr/Fid0kMTYa/WM=",
        sign_string_hmac(
            make_signing_key("kd94hf93k423kf44", "pfkkdhi9sl3r4s00"),
            long_base_string
        )
    );
}
