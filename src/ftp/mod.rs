// TODO: Switch to actor pattern. Implement audit logs.
pub mod auth;
use std::{path::PathBuf, sync::Arc};

use libunftp::ServerBuilder;
use tracing::instrument;
use unftp_sbe_fs::Filesystem;

use crate::config::FtpSettings;

#[instrument(level = "DEBUG", skip(root_path))]
pub async fn start_ftp_server(settings: FtpSettings, root_path: impl Into<PathBuf>) {
    let root_path = root_path.into();
    let authenticator = Arc::new(auth::AuthManager::new(&root_path));

    let server = ServerBuilder::with_authenticator(
        Box::new(move || Filesystem::new(root_path.clone())),
        authenticator,
    )
    .build()
    .await
    .unwrap();

    tracing::info!(
        "FTPS server alive at {}:{}",
        settings.interface,
        settings.port
    );
    server
        .listen(format!("{}:{}", settings.interface, settings.port))
        .await
        .unwrap();
}
