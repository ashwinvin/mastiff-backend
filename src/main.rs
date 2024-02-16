
pub mod config;
pub mod ftp;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));

    let ftp_server = ftp::create_ftp_server("/Users/ashwinvinod/projects/".to_string()).await;

    ftp_server.listen("127.0.0.1:2121").await;
}
