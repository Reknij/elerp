mod config;
mod custom_error;
pub mod db;
mod erp;
mod meta;
mod myhelper;
mod public_system;
mod services;
pub mod user_system;

use clap::Parser;
pub use tracing::info;
use tracing::warn;

use crate::{config::AppConfig, meta::MetaInfo};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();
    let meta = MetaInfo::parse();
    if !check_meta(&meta) {
        return;
    }
    let config = AppConfig::new(meta.clone()).await;
    let pool = db::init_db(&config, false).await.unwrap();
    match meta.cmd {
        meta::Commands::Update => {
            if db::update(pool).await {
                info!("Data updated!")
            } else {
                warn!("Nothing to update!")
            }
        }
        meta::Commands::Serve => {
            if db::update(pool.clone()).await {
                info!("Data updated!")
            }
            info!("Elerp starting..");
            services::serve(config, pool).await
        }
    }
}

fn check_meta(meta: &MetaInfo) -> bool {
    if !meta.data_path.is_dir() {
        warn!("`data-path is not directory or not found!`");
        false
    } else {
        true
    }
}
