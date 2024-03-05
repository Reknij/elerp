mod custom_error;
pub mod db;
mod erp;
mod meta;
mod myhelper;
mod public_system;
mod services;
mod config;
pub mod user_system;

pub use tracing::info;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::ERROR
        })
        .init();
    info!("Elerp starting..");
    services::serve().await;
}
