use elerp_common::{config::AppConfig, meta};
use tokio::fs;
pub use tracing::info;
use tracing::{error, warn};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) { tracing::Level::DEBUG } else { tracing::Level::INFO })
        .init();
    let meta = meta::MetaInfo::new();
    if !check_meta(&meta).await {
        return;
    }
    let config = AppConfig::new(meta.clone()).await;
    match meta.cmd {
        meta::Commands::Update => {
            if elerp_service::update(config).await {
                info!("Data updated!")
            } else {
                warn!("Nothing to update!")
            }
        }
        meta::Commands::Serve => {
            info!("Elerp starting..");
            elerp_service::serve(config).await
        }
    }
}

async fn check_meta(meta: &meta::MetaInfo) -> bool {
    if meta.data_path.exists() {
        if !meta.data_path.is_dir() {
            warn!("`data-path` is not the directory!");
            return false;
        }
    } else {
        warn!("`data-path` is not found or not directory. Will create the new one.");
        if let Err(err) = fs::create_dir(&meta.data_path).await {
            error!("Create the `data-path` failed: {err}")
        }
    }

    true
}
