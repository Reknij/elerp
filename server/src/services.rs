pub mod erp_service;
pub mod models;
pub mod user_service;
pub mod ws_service;

use crate::{config::AppConfig, erp::ERP, public_system::PublicSystem, user_system::UserSystem};
use axum::{
    extract::{FromRef, State},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tokio::fs;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::info;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub erp: ERP,
    pub us: UserSystem,
    pub ps: PublicSystem,
}

impl FromRef<State<AppState>> for AppState {
    fn from_ref(input: &State<AppState>) -> Self {
        input.0.clone()
    }
}

pub async fn serve(config: AppConfig, pool: sqlx::Pool<sqlx::Sqlite>) {
    info!("Using {:#?}", config);
    let ps = PublicSystem::new(pool, config.clone()).await;
    let state = AppState {
        erp: ERP::new(ps.clone()).await,
        us: UserSystem::new(ps.clone()).await,
        ps,
    };
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);
    //initial_erp(erp.clone()).await;
    let mut erp_openapi = erp_service::ApiDoc::openapi();
    let mut us_openapi = user_service::ApiDoc::openapi();
    erp_openapi.servers = Some(vec![utoipa::openapi::Server::new("/api/erp")]);
    us_openapi.servers = Some(vec![utoipa::openapi::Server::new("/api/us")]);

    let api_router = Router::new()
        .nest("/erp", erp_service::get_services())
        .nest("/us", user_service::get_services());
    let serve_dir = ServeDir::new(&config.web.dist)
        .not_found_service(ServeFile::new(&config.web.dist.join(&config.web.index)));
    let app = Router::new()
        .nest_service("/", serve_dir)
        .merge(
            RapiDoc::with_openapi("/crm-api-docs/openapi.json", erp_openapi).path("/rapidoc/erp"),
        )
        .merge(RapiDoc::with_openapi("/wms-api-docs/openapi.json", us_openapi).path("/rapidoc/us"))
        .nest("/api", api_router)
        .nest("/socket", ws_service::get_services())
        .with_state(state)
        .layer(cors);

    info!("Elerp will served at {}:{}..", config.host, config.port);

    let enable_tls = {
        if config.tls.self_tls {
            let subject_alt_names: &[_] = &[
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ];
            let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
            Some((
                cert.serialize_pem().unwrap().as_bytes().to_vec(),
                cert.serialize_private_key_pem().as_bytes().to_vec(),
            ))
        } else if config.tls.cert.is_some() && config.tls.key.is_some() {
            let cert_path = config.tls.cert.as_ref().unwrap();
            let key_path = config.tls.key.as_ref().unwrap();
            Some((
                fs::read(cert_path).await.expect("Can't read the cert path"),
                fs::read(key_path)
                    .await
                    .expect("Can't read the private key path"),
            ))
        } else {
            None
        }
    };
    if let Some((cert, key)) = enable_tls {
        let rustls_config = RustlsConfig::from_pem(cert, key).await.unwrap();
        axum_server::bind_rustls(
            format!("{}:{}", config.host, config.port).parse().unwrap(),
            rustls_config,
        )
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    } else {
        axum_server::bind(format!("{}:{}", config.host, config.port).parse().unwrap())
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    }
}
