use std::io::Write;

use url::Url;

use oagain::Result;
use oagain::{BasicConsumer, ETradePreset};

fn main() -> Result<()> {
    env_logger::init();

    let mut consumer = BasicConsumer::builder()
        .use_preset(ETradePreset)?
        .use_secrets_file("secrets.toml")?
        .use_save_file("save.toml")?
        .build()?;

    if !consumer.is_fully_authed() {
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
    }

    let response_str = consumer.get(&Url::parse("https://api.etrade.com/v1/accounts/list")?)?;
    println!("RESPONSE: {}", response_str);

    Ok(())
}
