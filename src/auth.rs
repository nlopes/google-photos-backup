use hyper::net::HttpsConnector;
use hyper::Client;
use hyper_rustls::TlsClient;
use yup_oauth2::{
    ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate, DiskTokenStorage, FlowType,
};

pub type LibraryAuthenticator =
    Authenticator<DefaultAuthenticatorDelegate, DiskTokenStorage, Client>;

use crate::config::Config;

pub fn authenticate(config: &Config) -> LibraryAuthenticator {
    let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
    let secret = ApplicationSecret {
        client_id: env!("GOOGLE_PHOTOS_BACKUP_CLIENT_ID").to_string(),
        client_secret: env!("GOOGLE_PHOTOS_BACKUP_CLIENT_SECRET").to_string(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec![
            "http://localhost".to_string(),
            "urn:ietf:wg:oauth:2.0:oob".to_string(),
        ],
        ..Default::default()
    };

    let storage = if let Some(token_path) = config.cache().join("credentials.json").to_str() {
        DiskTokenStorage::new(&token_path.to_string()).expect("credentials cache")
    } else {
        panic!("Could not setup disk storage");
    };

    Authenticator::new(
        &secret,
        DefaultAuthenticatorDelegate,
        client,
        storage,
        Some(FlowType::InstalledRedirect(8080)),
    )
}
