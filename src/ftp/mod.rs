pub mod auth;
use std::sync::Arc;

use libunftp::{Server, ServerBuilder};
use unftp_sbe_fs::Filesystem;

use self::auth::User;

pub async fn create_ftp_server(root_path: String) -> Server<Filesystem, User> {
    let authenticator = Arc::new(auth::AuthManager::new(&root_path));
    
    let server = ServerBuilder::with_authenticator(
        Box::new(move || unftp_sbe_fs::Filesystem::new(root_path.clone())),
        authenticator,
    );
    server.build().await.unwrap()
}
