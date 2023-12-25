#[derive(Debug, Default)]
pub enum ConsumerState {
    #[default]
    /// Initial state. Ready to retrieve a request token from the server.
    NoAuth,

    /// The request token has been retrieved. Ready to produce a user auth URL.
    RequestToken {
        request_token: String,
        token_secret: String,
    },

    /// The user has provided a verification code. Ready to retrieve the Access token.
    UserAuth {
        request_token: String,
        token_secret: String,
        verification_code: String,
    },

    /// Access token retrieved. Ready for API requests.
    FullAuth {
        access_token: String,
        token_secret: String,
    },
}

impl ConsumerState {
    pub fn token(&self) -> Option<&str> {
        match self {
            ConsumerState::RequestToken { request_token, .. }
            | ConsumerState::UserAuth { request_token, .. } => Some(request_token),
            ConsumerState::FullAuth { access_token, .. } => Some(access_token),
            _ => None,
        }
    }

    pub fn token_secret(&self) -> Option<&str> {
        match self {
            ConsumerState::RequestToken { token_secret, .. }
            | ConsumerState::UserAuth { token_secret, .. }
            | ConsumerState::FullAuth { token_secret, .. } => Some(token_secret),
            ConsumerState::NoAuth => None,
        }
    }

    pub fn verification_code(&self) -> Option<&str> {
        match self {
            ConsumerState::UserAuth {
                verification_code, ..
            } => Some(verification_code),
            _ => None,
        }
    }
}
