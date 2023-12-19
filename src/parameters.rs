use crate::pencoding::{decode_str, encode_param};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ParamPair {
    pub name: String,
    pub value: Option<String>,
}

impl ParamPair {
    pub fn single(name: impl Into<String>) -> ParamPair {
        ParamPair {
            name: name.into(),
            value: None,
        }
    }

    pub fn pair(name: impl Into<String>, value: impl Into<String>) -> ParamPair {
        ParamPair {
            name: name.into(),
            value: Some(value.into()),
        }
    }

    pub fn to_wrapped_string(&self) -> String {
        format!(
            "{}=\"{}\"",
            encode_param(&self.name),
            self.value.as_ref().map(encode_param).unwrap_or_default()
        )
    }
}

impl ToString for ParamPair {
    fn to_string(&self) -> String {
        format!(
            "{}={}",
            encode_param(&self.name),
            self.value.as_ref().map(encode_param).unwrap_or_default()
        )
    }
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

    #[test]
    fn params_to_string() {
        // Regular
        assert_eq!(
            "dragon=tiamat",
            ParamPair {
                name: "dragon".to_string(),
                value: Some("tiamat".to_string())
            }
            .to_string()
        );

        // null value
        assert_eq!(
            "salamander=",
            ParamPair {
                name: "salamander".to_string(),
                value: None
            }
            .to_string()
        )
    }
}
