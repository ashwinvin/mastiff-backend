#![allow(unused_must_use)]

use std::sync::Arc;

use clap::Parser;
use mastiff_backend::{
    cli, config, ftp,
    managers::{recipe::RecipeManager, Managers},
    routes::initialise_routes,
};
use tracing_panic::panic_hook;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    let cli_args = cli::Cli::parse();
    let settings = config::Settings::new(&cli_args.config_path).unwrap();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        panic_hook(panic_info);
        prev_hook(panic_info);
    }));

    let ftp_settings = settings.ftp.clone();
    let data_dir = settings.container_data_directory.clone();

    // TODO: Move the tokio spawn inside
    tokio::spawn(async move {
        ftp::start_ftp_server(ftp_settings, data_dir).await;
    });

    let managers = Managers::new(&settings).await;

    let router = initialise_routes(managers);
    let listener =
        tokio::net::TcpListener::bind((settings.rest_api.interface, settings.rest_api.port))
            .await
            .unwrap();

    axum::serve(listener, router).await.unwrap();
}
