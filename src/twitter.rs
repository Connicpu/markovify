use std::mem::replace;
use hyper::error::Result as HyperResult;
use hyper::method::Method;
use hyper::client::Response;
use tweetust;
use tweetust::conn::Parameter;

pub struct TwitterTrainer {
    consumer_key: String,
    consumer_secret: String,
    authentication: Option<AuthType>,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct SaveData {
    consumer_key: String,
    consumer_secret: String,
    app_token: Option<String>,
    user_token: Option<(String, String)>,
}

impl TwitterTrainer {
    pub fn new(consumer_key: String, consumer_secret: String) -> TwitterTrainer {
        TwitterTrainer {
            consumer_key: consumer_key,
            consumer_secret: consumer_secret,
            authentication: None,
        }
    }

    pub fn from_data(data: SaveData) -> TwitterTrainer {
        let auth = match (data.app_token, data.user_token) {
            (_, Some(access)) => {
                Some(AuthType::UserAuthenticated(tweetust::OAuthAuthenticator::new(
                    &data.consumer_key,
                    &data.consumer_secret,
                    &access.0,
                    &access.1,
                )))
            },
            (Some(token), None) => {
                Some(AuthType::AppOnly(tweetust::ApplicationOnlyAuthenticator(token)))
            },
            _ => None,
        };

        TwitterTrainer {
            consumer_key: data.consumer_key,
            consumer_secret: data.consumer_secret,
            authentication: auth,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authentication.is_some()
    }

    pub fn authenticate_app(&mut self) -> Result<(), tweetust::TwitterError> {
        let request = tweetust::oauth2::token(&self.consumer_key, &self.consumer_secret);
        let result = try!(request.execute()).object;
        let auth = tweetust::ApplicationOnlyAuthenticator(result.access_token);
        self.authentication = Some(AuthType::AppOnly(auth));
        Ok(())
    }

    pub fn get_client(&self) -> Option<tweetust::TwitterClient<AuthType>> {
        match self.authentication {
            Some(ref auth) => Some(tweetust::TwitterClient::new(auth.clone())),
            None => None,
        }
    }

    pub fn get_save_data(&self) -> SaveData {
        SaveData {
            consumer_key: self.consumer_key.clone(),
            consumer_secret: self.consumer_secret.clone(),
            app_token: if let Some(AuthType::AppOnly(ref auth)) = self.authentication {
                Some(auth.0.clone())
            } else {
                None
            },
            user_token: if let Some(AuthType::UserAuthenticated(ref auth)) = self.authentication {
                Some((auth.access_token.clone(), auth.access_token_secret.clone()))
            } else {
                None
            },
        }
    }

    pub fn load_save_data(&mut self, data: SaveData) {
        replace(self, TwitterTrainer::from_data(data));
    }
}

#[derive(Clone)]
pub enum AuthType {
    AppOnly(tweetust::ApplicationOnlyAuthenticator),
    UserAuthenticated(tweetust::OAuthAuthenticator),
}

impl tweetust::conn::Authenticator for AuthType {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter])
            -> HyperResult<Response> {
        use self::AuthType::*;
        match self {
            &AppOnly(ref auth) => auth.send_request(method, url, params),
            &UserAuthenticated(ref auth) => auth.send_request(method, url, params),
        }
    }
}
