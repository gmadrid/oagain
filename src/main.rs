use std::fs::File;
use std::io::{Read, Write};

use toml::Table;

use oagain::{BasicConsumer, ETradePreset};
use oagain::{OagainError, Result};

fn main() -> Result<()> {
    env_logger::init();

    let (key, secret) = read_key_and_secret()?;

    let mut consumer = BasicConsumer::builder()
        .use_preset(ETradePreset)?
        .set_consumer_key(key)
        .set_consumer_secret(secret)
        .build()?;

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
    consumer.set_verification_code(code.trim())?;

    consumer.retrieve_access_token()?;

    Ok(())
}

fn read_key_and_secret() -> Result<(String, String)> {
    let mut s = String::new();
    let mut f = File::open("secrets.toml")?;
    f.read_to_string(&mut s)?;
    let table = s.parse::<Table>()?;
    let key = table.get("token");
    if key.is_none() {
        return Err(OagainError::MissingConsumerToken("in secrets file"));
    }
    let secret = table.get("secret");
    if secret.is_none() {
        return Err(OagainError::MissingConsumerSecret("in secrets file"));
    }
    Ok((
        key.and_then(|k| k.as_str()).unwrap().to_string(),
        secret.and_then(|v| v.as_str()).unwrap().to_string(),
    ))
}
