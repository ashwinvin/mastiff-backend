use std::fmt::Debug;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use libunftp::auth::Credentials;
use libunftp::auth::UserDetail;
use libunftp::auth::{AuthenticationError, Authenticator};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("An unexpected error occured while looking up user info.")]
    Unknown,
    #[error("Only password based authentication is supported")]
    UnsupportedMethod,
}

impl From<AuthError> for AuthenticationError {
    fn from(value: AuthError) -> Self {
        AuthenticationError::ImplPropagated(value.to_string(), Some(Box::new(value)))
    }
}

#[derive(Debug)]
pub struct User {
    username: String,
    root_path: PathBuf,
}

impl User {
    pub fn new<P: Into<PathBuf>>(username: &str, root_path: P) -> Self {
        User {
            username: username.to_string(),
            root_path: root_path.into(),
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl UserDetail for User {
    fn account_enabled(&self) -> bool {
        true
    }

    fn home(&self) -> Option<&Path> {
        Some(&self.root_path)
    }
}
#[derive(Debug)]
pub struct AuthManager {
    container_root: PathBuf,
}

impl AuthManager {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        AuthManager {
            container_root: root.into(),
        }
    }
}

#[async_trait::async_trait]
impl Authenticator<User> for AuthManager {
    async fn authenticate(
        &self,
        username: &str,
        creds: &Credentials,
    ) -> Result<User, AuthenticationError> {
        tracing::info!("{:?}", creds);
        if let Some(password) = &creds.password {
            return Ok(User {
                username: "meower".to_string(),
                root_path: self.container_root.join("anvesh"),
            });
        }
        // I have no idea what this is, but it works. thanks rust analyzer
        Err(Into::<AuthenticationError>::into(
            AuthError::UnsupportedMethod,
        ))
    }
}
