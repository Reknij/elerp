use std::path::PathBuf;

use serde::Deserialize;
use tokio::fs;
use tracing::warn;

use crate::{meta::MetaInfo, myhelper::i64_safe_max};

#[derive(Debug, Deserialize)]
struct AppConfigInternal {
    pub host: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub web: Web,
    #[serde(default)]
    pub limit: Limit,
    #[serde(default)]
    pub tls: TLS,
    #[serde(default)]
    pub ws: WS,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_path: PathBuf,
    pub web: Web,
    pub host: String,
    pub port: u16,
    pub limit: Limit,
    pub tls: TLS,
    pub ws: WS,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Limit {
    #[serde(default = "i64_safe_max")]
    pub users: i64,
    #[serde(default = "i64_safe_max")]
    pub areas: i64,
    #[serde(default = "i64_safe_max")]
    pub persons: i64,
    #[serde(default = "i64_safe_max")]
    pub warehouses: i64,
    #[serde(default = "i64_safe_max")]
    pub sku_categories: i64,
    #[serde(default = "i64_safe_max")]
    pub skus: i64,
    #[serde(default = "i64_safe_max")]
    pub order_categories: i64,
    #[serde(default = "i64_safe_max")]
    pub orders: i64,
    #[serde(default = "i64_safe_max")]
    pub guest_orders: i64,
    #[serde(default = "i64_safe_max")]
    pub order_payments: i64,
    #[serde(default = "i64_safe_max")]
    pub statistics: i64,
}

impl Default for Limit {
    fn default() -> Self {
        let safe_max = i64_safe_max();
        Limit {
            users: safe_max,
            areas: safe_max,
            persons: safe_max,
            warehouses: safe_max,
            sku_categories: safe_max,
            skus: safe_max,
            order_categories: safe_max,
            orders: safe_max,
            guest_orders: safe_max,
            order_payments: safe_max,
            statistics: safe_max,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TLS {
    pub self_tls: bool,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Web {
    pub dist: PathBuf,
    pub index: String,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            dist: "./dist".into(),
            index: "index.html".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct WS {
    pub ping: Option<usize>,
}

impl AppConfigInternal {
    pub fn new(meta: MetaInfo) -> Self {
        let web = Web::default();
        Self {
            host: meta.host,
            port: meta.port,
            web: Web {
                dist: meta.web_dist.unwrap_or(web.dist),
                index: meta.web_index.unwrap_or(web.index),
            },
            limit: Limit::default(),
            tls: TLS {
                self_tls: meta.self_tls,
                cert: meta.tls_cert,
                key: meta.tls_key,
            },
            ws: WS { ping: meta.ping },
        }
    }
    pub fn overwrite(&mut self, meta: MetaInfo) {
        if let Some(host) = meta.host {
            self.host = Some(host);
        }
        if let Some(port) = meta.port {
            self.port = Some(port);
        }
        if let Some(cert) = meta.tls_cert {
            self.tls.cert = Some(cert);
        }
        if let Some(key) = meta.tls_key {
            self.tls.key = Some(key);
        }
        if meta.self_tls {
            self.tls.self_tls = meta.self_tls;
        }
        if let Some(dist) = meta.web_dist {
            self.web.dist = dist;
        }
        if let Some(index) = meta.web_index {
            self.web.index = index;
        }
    }
}

impl AppConfig {
    fn from_internal(data_path: PathBuf, value: AppConfigInternal) -> Self {
        Self {
            web: Web {
                dist: value.web.dist,
                index: value.web.index,
            },
            host: value.host.unwrap_or("0.0.0.0".to_owned()),
            port: value.port.unwrap_or(3344),
            limit: value.limit,
            tls: value.tls,
            ws: value.ws,
            data_path,
        }
    }
    pub async fn new(meta: MetaInfo) -> Self {
        let config_path = meta.data_path.join("config.toml");
        let data_path = meta.data_path.clone();
        if config_path.is_file() {
            match fs::read_to_string(&config_path).await {
                Ok(buf) => match toml::from_str::<AppConfigInternal>(&buf) {
                    Ok(mut internal) => {
                        internal.overwrite(meta);
                        Self::from_internal(data_path, internal)
                    }
                    Err(err) => {
                        warn!("Deserialize config structure failed: {err}");
                        warn!("Will using the default config...");
                        Self::from_internal(meta.data_path.clone(), AppConfigInternal::new(meta))
                    }
                },
                Err(err) => {
                    warn!("Read the config file error: {err}");
                    warn!("Will using the default config...");
                    Self::from_internal(meta.data_path.clone(), AppConfigInternal::new(meta))
                }
            }
        } else {
            Self::from_internal(meta.data_path.clone(), AppConfigInternal::new(meta))
        }
    }
}
