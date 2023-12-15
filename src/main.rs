use oagain::{BasicConsumer, Config, OagainError, Result};
use std::fs::File;
use std::io::Read;
use toml::Table;
use url::Url;

fn main() -> oagain::Result<()> {
    let config = Config {
        request_token_url: Url::parse("https://api.etrade.com/oauth/request_token")?,
        user_authorization_url: Url::parse("https://us.etrade.com/e/t/etws/authorize")?,
        access_token_url: Url::parse("https://api.etrade.com/oauth/access_token")?,
    };

    let (key, secret) = read_key_and_secret()?;

    // TODO: these values are meaningless.
    let mut consumer = BasicConsumer::new(key, secret, config)?;

    consumer.obtain_request_token()?;

    Ok(())
}

fn read_key_and_secret() -> Result<(String, String)> {
    let mut s = String::new();
    let mut f = File::open("secrets.toml")?;
    f.read_to_string(&mut s)?;
    let table = s.parse::<Table>()?;
    let key = table.get("token");
    if key.is_none() {
        return Err(OagainError::MissingConsumerToken);
    }
    let secret = table.get("secret");
    if secret.is_none() {
        return Err(OagainError::MissingConsumerSecret);
    }
    Ok((
        key.and_then(|k| k.as_str()).unwrap().to_string(),
        secret.and_then(|v| v.as_str()).unwrap().to_string(),
    ))
}
