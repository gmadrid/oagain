use itertools::{sorted, Itertools};
use url::Url;

use crate::constants::OAUTH_SIGNATURE_PARAM_NAME;
use crate::parameters::ParamPair;

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
fn normalize_request_parameters(pairs: &[ParamPair]) -> String {
    sorted(pairs)
        .filter(|p| p.name != OAUTH_SIGNATURE_PARAM_NAME)
        .map(|pair| pair.to_string())
        .join("&")
}

pub fn concat_request_elements(method: &str, url: &Url, pairs: &[ParamPair]) -> String {
    format!(
        "{}&{}&{}",
        method,
        construct_request_url(url),
        normalize_request_parameters(pairs)
    )
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::{concat_request_elements, construct_request_url, normalize_request_parameters};

    #[test]
    fn concat_request() {
        assert_eq!(
            "POST&http://example.com/the_path&four=4&one=1%20afterspace&three=3&two=2",
            concat_request_elements(
                "POST",
                &Url::parse("http://example.com/the_path").unwrap(),
                &[
                    "one=1 afterspace".into(),
                    "two=2".into(),
                    "three=3".into(),
                    "four=4".into()
                ]
            )
        )
    }

    #[test]
    fn basic_params() {
        assert_eq!(
            "a=1&c=hi%20there&f=25&f=50&f=a&z=p&z=t",
            normalize_request_parameters(&[
                "a=1".into(),
                "c=hi there".into(),
                "f=50".into(),
                "f=25".into(),
                "f=a".into(),
                "z=p".into(),
                "z=t".into()
            ])
        );
    }

    #[test]
    fn filter_oauth_signature() {
        assert_eq!(
            "f=a&oauth_signature_one=1&oauth_signature_three=25&oauth_signature_two=hi%20there&z=p&z=t",
            normalize_request_parameters(&[
                "oauth_signature_one=1".into(),
                "oauth_signature_two=hi there".into(),
                "oauth_signature=50".into(),
                "oauth_signature_three=25".into(),
                "f=a".into(),
                "z=p".into(),
                "z=t".into()
            ])
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
