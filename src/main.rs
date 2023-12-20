use oagain::{BasicConsumer, Config, OagainError, Result};
use std::fs::File;
use std::io::{Read, Write};
use toml::Table;

fn main() -> oagain::Result<()> {
    let config = Config::new(
        "https://api.etrade.com/oauth/request_token",
        "https://us.etrade.com/e/t/etws/authorize",
        "https://api.etrade.com/oauth/access_token",
    )?
    // Note that the eTrade documentation (https://apisb.etrade.com/docs/api/authorization/authorize.html)
    // is not correct. The key and token param names are non-standard. They are listed in the docs
    // incorrectly, but the Example is correct.
    .set_user_auth_param_names("key", "token");

    let (key, secret) = read_key_and_secret()?;
    let mut consumer = BasicConsumer::new(key, secret, config)?;
    consumer.retrieve_request_token()?;

    let url = consumer.make_user_auth_url()?;
    println!(
        "Go to the following URL and follow the instructions:\n\n    {}\n\n",
        url
    );

    print!("Input the verification code received from the server: ");
    std::io::stdout().flush()?;
    let mut code: String = Default::default();
    std::io::stdin().read_line(&mut code)?;
    consumer.set_verification_code(code.trim());

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
