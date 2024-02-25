use std::{net::IpAddr, ops::Range, path::PathBuf};

use config::{Config, ConfigError, File};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct FtpSettings {
    /// The port to listen for ftp connections.
    pub port: u16,
    /// The interface to listen on for ftp connection.
    pub interface: IpAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiSettings {
    /// The port to listen for ftp connections.
    pub port: u16,
    /// The interface to listen on for ftp connection.
    pub interface: IpAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PanelSettings {
    /// Panel Url
    url: Url,
    /// Maxmum no of seconds to wait before retrying a request.
    max_timeout: u8,
    /// Maximum no of times to retry sending a request before raising an exception.
    max_retries: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerManagerSettings {
    /// Range of ports that can be allocated to containers
    pub container_port_range: Range<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// The path where all the container data are stored.
    pub container_data_directory: PathBuf,
    /// Range of ports that can be allocated to containers
    pub container_manager: ContainerManagerSettings,
    /// The path where all the recipes are stored.
    pub recipe_directory: PathBuf,
    /// FTP configuration
    pub ftp: FtpSettings,
    /// Panel configuration
    pub panel: PanelSettings,
    /// Rest API configuration
    pub rest_api: ApiSettings,
}

impl Settings {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        tracing::debug!("Loading config from: {}", config_path);
        let config = Config::builder()
            .add_source(File::with_name(config_path).required(true))
            .build()?;

        tracing::debug!("Config loaded: {:#?}", config);
        config.try_deserialize()
    }
}
