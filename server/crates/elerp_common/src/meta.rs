use std::path::PathBuf;

use clap::{arg, command, ArgAction, Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct MetaInfo {
    /// Hostname to host server. Default `http://127.0.0.1` or `https://127.0.0.1`.
    #[arg(long)]
    pub host: Option<String>,

    /// Port to host server. Default `3344`
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Path to an SSL/TLS certificate to serve with HTTPS.
    #[arg(long)]
    pub tls_cert: Option<PathBuf>,

    /// Path to the SSL/TLS certificate's private key.
    #[arg(long)]
    pub tls_key: Option<PathBuf>,

    /// Enable self signed TLS
    #[arg(long, action=ArgAction::SetTrue)]
    pub self_tls: bool,

    /// Enable ping for websocket. (seconds)
    #[arg(long)]
    pub ping: Option<usize>,

    /// Server data save path. Include database files.
    #[arg(short, long)]
    pub data_path: PathBuf,

    /// Website distribution path. Default `./dist`
    #[arg(long)]
    pub web_dist: Option<PathBuf>,

    /// Website index file name. Default `index.html`
    #[arg(long)]
    pub web_index: Option<String>,

    #[command(subcommand)]
    pub cmd: Commands,
}

impl MetaInfo {
    pub fn new() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Update,
    Serve,
}
