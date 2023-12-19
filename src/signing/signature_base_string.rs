use itertools::Itertools;
use url::Url;

use crate::constants::OAUTH_SIGNATURE_PARAM_NAME;
use crate::parameters::ParamPair;
use crate::pencoding::encode_param;

fn construct_request_url(url: &Url) -> String {
    if url.query().is_some() || url.fragment().is_some() {
        let mut other = url.clone();
        other.set_fragment(None);
        other.set_query(None);
        other.to_string()
    } else {
        url.to_string()
    }
}

// pairs of (name: &str, value: &str)
pub(crate) fn normalize_request_parameters(pairs: impl Iterator<Item = ParamPair>) -> String {
    pairs
        .sorted()
        .filter(|p| p.name != OAUTH_SIGNATURE_PARAM_NAME)
        .map(|pair| pair.to_string())
        .join("&")
}

pub fn concat_request_elements(
    method: &str,
    url: &Url,
    pairs: impl Iterator<Item = ParamPair>,
) -> String {
    format!(
        "{}&{}&{}",
        encode_param(method),
        encode_param(construct_request_url(url)),
        encode_param(normalize_request_parameters(pairs))
    )
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::{concat_request_elements, construct_request_url, normalize_request_parameters};

    #[test]
    fn concat_request() {
        assert_eq!(
            "POST&http%3A%2F%2Fexample.com%2Fthe_path&four%3D4%26one%3D1%2520afterspace%26three%3D3%26two%3D2",
            concat_request_elements(
                "POST",
                &Url::parse("http://example.com/the_path").unwrap(),
                [
                    "one=1 afterspace".into(),
                    "two=2".into(),
                    "three=3".into(),
                    "four=4".into()
                ].into_iter()
            )
        )
    }

    #[test]
    fn basic_params() {
        assert_eq!(
            "a=1&c=hi%20there&f=25&f=50&f=a&z=p&z=t",
            normalize_request_parameters(
                [
                    "a=1".into(),
                    "c=hi there".into(),
                    "f=50".into(),
                    "f=25".into(),
                    "f=a".into(),
                    "z=p".into(),
                    "z=t".into()
                ]
                .into_iter()
            )
        );
    }

    #[test]
    fn filter_oauth_signature() {
        assert_eq!(
            "f=a&oauth_signature_one=1&oauth_signature_three=25&oauth_signature_two=hi%20there&z=p&z=t",
            normalize_request_parameters([
                "oauth_signature_one=1".into(),
                "oauth_signature_two=hi there".into(),
                "oauth_signature=50".into(),
                "oauth_signature_three=25".into(),
                "f=a".into(),
                "z=p".into(),
                "z=t".into()
            ].into_iter())
        );
    }

    #[test]
    fn strip_query_and_fragment() {
        let url = Url::parse("http://www.example.com/has_query?q=cat&ref=home#fragment").unwrap();

        // Ensure that the query and fragment are in the original URL.
        assert_eq!("fragment", url.fragment().unwrap());
        assert_eq!("q=cat&ref=home", url.query().unwrap());

        let base = construct_request_url(&url);
        assert_eq!("http://www.example.com/has_query", base);
    }

    #[test]
    fn no_eighty() {
        let url = Url::parse("http://www.example.com:80/foobar").unwrap();

        assert_eq!("http", url.scheme());
        assert_eq!("www.example.com", &url.host().unwrap().to_string());
        assert_eq!("/foobar", url.path());
        assert_eq!(None, url.port());

        // Show that the default port, 80, is not included in the string rep of the URL.
        // Per Spec 9.1.2
        assert_eq!("http://www.example.com/foobar", &url.to_string())
    }

    #[test]
    fn no_443() {
        let url = Url::parse("https://www.example.com:443/foobar").unwrap();

        assert_eq!("https", url.scheme());
        assert_eq!("www.example.com", &url.host().unwrap().to_string());
        assert_eq!("/foobar", url.path());
        assert_eq!(None, url.port());

        // Show that the default port, 80, is not included in the string rep of the URL.
        // Per Spec 9.1.2
        assert_eq!("https://www.example.com/foobar", &url.to_string())
    }
}
