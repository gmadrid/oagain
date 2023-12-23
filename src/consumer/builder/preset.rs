use crate::consumer::builder::Builder;
use crate::Result;
use url::Url;

pub trait Preset {
    fn setup_builder(self, builder: Builder) -> Result<Builder>;
}

pub struct ETradePreset;

impl Preset for ETradePreset {
    fn setup_builder(self, builder: Builder) -> Result<Builder> {
        Ok(builder
            .set_request_token_url(Url::parse("https://api.etrade.com/oauth/request_token")?)
            .set_user_authorization_url(Url::parse("https://us.etrade.com/e/t/etws/authorize")?)
            .set_access_token_url(Url::parse("https://api.etrade.com/oauth/access_token")?)
            .set_user_auth_key_param_name("key")
            .set_user_auth_token_param_name("token"))
    }
}