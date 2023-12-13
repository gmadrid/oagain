use crate::pencoding::decode_str;
use crate::Result;

#[derive(Debug, Eq, PartialEq)]
pub struct ParamPair {
    name: String,
    value: Option<String>,
}

impl From<&str> for ParamPair {
    fn from(value: &str) -> Self {
        // TODO: better error checking in here. (Really only case that value=='' or has 2+ '='.
        let pieces = value.split('=').collect::<Vec<_>>();
        if pieces.len() == 1 {
            ParamPair {
                name: decode_str(pieces[0]),
                value: None,
            }
        } else if pieces.len() == 2 {
            ParamPair {
                name: decode_str(pieces[0]),
                value: Some(decode_str(pieces[1])),
            }
        } else {
            panic!("Panicking due to invalid input string.")
        }
    }
}

pub fn decode_params_string(s: impl AsRef<str>) -> Vec<ParamPair> {
    s.as_ref().split('&').map(ParamPair::from).collect()
}

#[cfg(test)]
mod test {
    use crate::parameters::{decode_params_string, ParamPair};

    #[test]
    fn basic_test() {
        assert_eq!(
            vec![
                ParamPair {
                    name: "oauth_token".to_string(),
                    value: Some("ab3cd9j4ks73hf7g".to_string())
                },
                ParamPair {
                    name: "oauth_token_secret".to_string(),
                    value: Some("xyz4992k83j47x0b".to_string())
                },
                ParamPair {
                    name: "dummy_param".to_string(),
                    value: None
                }
            ],
            decode_params_string(
                "oauth_token=ab3cd9j4ks73hf7g&oauth_token_secret=xyz4992k83j47x0b&dummy_param"
            )
        )
    }

    #[test]
    fn decoding_test() {
        assert_eq!(
            vec![
                ParamPair {
                    name: "foo bar".to_string(),
                    value: Some("quux".to_string()),
                },
                ParamPair {
                    name: "¡Andale!".to_string(),
                    value: None
                }
            ],
            decode_params_string("foo%20bar=quux&%C2%A1Andale%21")
        );
    }
}
