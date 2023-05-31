use tracing::{error, info};
use std::{path::PathBuf, fs::read};

#[derive(Debug, clap::Parser, serde::Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    #[serde(skip_serializing)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    #[serde(skip_serializing)]
    pub private_key: Option<PathBuf>,

    #[arg(long)]
    #[serde(skip_serializing)]
    pub certificate: Option<PathBuf>,

    #[arg(short, long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub port: Option<u16>,

    #[arg(long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub proxy_port: Option<u16>,

    #[command(flatten)]
    #[serde(skip_serializing)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

impl Args {
    pub const CONFIG_FILENAME: &'static str = "config.toml";
    pub const PRIVATE_KEY_FILENAME: &'static str = "ca.key";
    pub const CERTIFICATE_FILENAME: &'static str = "ca.cer";

    fn get_config_path(name: &str, path: &Option<PathBuf>) -> PathBuf {
        if let Some(path) = path {
            return path.clone();
        }
        let mut folder = match directories::ProjectDirs::from("dev", "edger", "hub") {
            Some(proj) => proj.config_dir().to_path_buf(),
            None => PathBuf::new(),
        };
        folder.push(name);
        folder
    }
    pub fn config_path(&self) -> PathBuf {
        Self::get_config_path(Self::CONFIG_FILENAME, &self.config)
    }
    fn inject_env_from_file(key: &str, path: &PathBuf) {
        match read(path) {
            Ok(content) => {
                match std::str::from_utf8(&content) {
                    Ok(content) => {
                        info!(key, path = path.to_str(), "inject env from file");
                        std::env::set_var(key, content.to_string());
                    },
                    Err(err) => {
                        error!(key, path = path.to_str(), error = err.to_string(), "inject env from file failed when converting file content");
                    }
                }
            },
            Err(err) => {
                error!(key, path = path.to_str(), error = err.to_string(), "inject env from file failed when reading file");
            }
        }
    }
    pub fn inject_envs(&self) {
        Self::inject_env_from_file(crate::Config::ENV_PRIVATE_KEY,
            &Self::get_config_path(Self::PRIVATE_KEY_FILENAME, &self.private_key));
        Self::inject_env_from_file(crate::Config::ENV_CERTIFICATE,
            &Self::get_config_path(Self::CERTIFICATE_FILENAME, &self.certificate));
    }
}