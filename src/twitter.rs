use hyper::error::Result;
use hyper::method::Method;
use hyper::client::Response;
use tweetust;
use tweetust::conn::Parameter;

pub struct TwitterTrainer {
    consumer_key: String,
    consumer_secret: String,
    authentication: Option<AuthType>,
}

impl TwitterTrainer {
    pub fn new(consumer_key: String, consumer_secret: String) -> TwitterTrainer {
        TwitterTrainer {
            consumer_key: consumer_key,
            consumer_secret: consumer_secret,
            authentication: None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authentication.is_some()
    }

    pub fn authenticate_app(&self) -> Result<()> {
        tweetust::oauth2::token(&self.consumer_key, &self.consumer_secret);
        Ok(())
    }
}

#[derive(Clone)]
pub enum AuthType {
    AppOnly(tweetust::ApplicationOnlyAuthenticator),
    UserAuthenticated(tweetust::OAuthAuthenticator),
}

impl tweetust::conn::Authenticator for AuthType {

    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> Result<Response> {
        use self::AuthType::*;
        match self {
            &AppOnly(ref auth) => auth.send_request(method, url, params),
            &UserAuthenticated(ref auth) => auth.send_request(method, url, params),
        }
    }
}
