/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

use crate::settings::tls_parser::{CipherSuite, KxGroup, ProtocolVersion};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Settings {
    // "inline" allows showing only the "Settings" schema in the Swagger UI without the other structs since they are never returned by themselves.
    // And we also don't need to add the other structs to components(schemas(...)) in main.rs
    #[schema(inline)]
    pub credentials: Credentials,

    #[serde(default)]
    #[schema(inline)]
    pub metrics: Metrics,

    #[serde(default)]
    #[schema(inline)]
    pub applications: Applications,

    #[serde(default)]
    #[schema(inline)]
    pub web: Web,

    #[serde(default)]
    #[schema(inline)]
    pub openapi: OpenApi,

    #[serde(default)]
    #[schema(inline)]
    pub tls: Tls,

    #[serde(default)]
    #[schema(inline)]
    pub debug: Debug,
}

fn hide_client_secret<T, S: Serializer>(_value: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str("******")
}

fn verify_credential_present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "" | "..." => Err(serde::de::Error::custom("no credential found")),
        _ => Ok(value),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Credentials {
    #[serde(deserialize_with = "verify_credential_present")]
    pub tenant_id: String,

    #[serde(deserialize_with = "verify_credential_present")]
    pub client_id: String,

    #[serde(serialize_with = "hide_client_secret")] // Do not leak the client secret when exposing our credentials on an API endpoint
    #[serde(deserialize_with = "verify_credential_present")]
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Metrics {
    #[serde(with = "humantime_serde")]
    #[schema(example = "30m")]
    pub prune_interval: Option<Duration>,

    #[serde(with = "humantime_serde")]
    #[schema(example = "1m")]
    pub refresh_interval: Duration,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            prune_interval: Some(Duration::from_secs(60 * 30)),
            refresh_interval: Duration::from_secs(60),
        }
    }
}

/// Enforce that the given value is within the supported range
fn de_results_per_page<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    let value = u16::deserialize(deserializer)?;
    if (1..=999).contains(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("value not in range 1..=999"))
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Applications {
    pub enabled: bool,

    #[serde(with = "humantime_serde")]
    #[schema(value_type = String, example = "15m", default = "15m")]
    pub cache_refresh_interval: Duration,

    pub url: String,

    #[serde(deserialize_with = "de_results_per_page")]
    #[schema(minimum = 1, maximum = 999)]
    pub results_per_page: u16,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_refresh_interval: Duration::from_secs(60 * 15),
            url: "https://graph.microsoft.com/v1.0/applications".into(),
            results_per_page: 999,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Web {
    #[schema(value_type = String)]
    pub listen_address: SocketAddr,

    #[schema(value_type = Option<String>)]
    pub cert_file: Option<PathBuf>,

    #[schema(value_type = Option<String>)]
    pub key_file: Option<PathBuf>,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:9081".parse().expect("hardcoded value must parse"),
            cert_file: Default::default(),
            key_file: Default::default(),
        }
    }
}

fn check_url<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let url = String::deserialize(d)?;

    match url.as_str() {
        "" | "/" => Err(serde::de::Error::custom(r#"url cannot be empty or "/""#)),
        s if !s.starts_with('/') => Err(serde::de::Error::custom(r#"url must start with "/""#)),
        _ => Ok(url),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct OpenApi {
    pub enabled: bool,

    #[serde(deserialize_with = "check_url")]
    pub docs_url: String,

    #[serde(deserialize_with = "check_url")]
    pub swagger_ui_url: String,
}

impl Default for OpenApi {
    fn default() -> Self {
        Self {
            enabled: true,
            docs_url: "/openapi.json".into(),
            swagger_ui_url: "/swagger".into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Tls {
    #[schema(inline)]
    pub cipher_suites: Vec<CipherSuite>,

    #[schema(inline)]
    pub key_exchange_groups: Vec<KxGroup>,

    #[schema(inline)]
    pub protocol_versions: Vec<ProtocolVersion>,
}

impl Tls {
    pub fn rustls_cipher_suites(&self) -> Vec<rustls::SupportedCipherSuite> {
        self.cipher_suites.iter().copied().map(From::from).collect()
    }

    pub fn rustls_kx_groups(&self) -> Vec<&'static rustls::SupportedKxGroup> {
        self.key_exchange_groups.iter().copied().map(From::from).collect()
    }

    pub fn rustls_protocol_versions(&self) -> Vec<&'static rustls::SupportedProtocolVersion> {
        self.protocol_versions.iter().copied().map(From::from).collect()
    }
}

impl Default for Tls {
    fn default() -> Self {
        use CipherSuite::*;
        use KxGroup::*;
        use ProtocolVersion::*;
        Self {
            cipher_suites: vec![
                TLS13_AES_256_GCM_SHA384,
                TLS13_AES_128_GCM_SHA256,
                TLS13_CHACHA20_POLY1305_SHA256,
                TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
            ],
            key_exchange_groups: vec![X25519, SECP256R1, SECP384R1],
            protocol_versions: vec![TLS13, TLS12],
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Debug {
    pub no_verify_tls: bool,
}

pub fn parse() -> Settings {
    let settings_env_var = "AZURE_APP_EXPORTER_SETTINGS_PATH";
    let default_settings_path = "/etc/azure_app_exporter/settings.toml";

    let settings_path = std::env::var(settings_env_var).unwrap_or_else(|_| {
        tracing::warn!("no {settings_env_var} env var set, defaulting to {default_settings_path}");
        default_settings_path.into()
    });

    let settings_contents = std::fs::read_to_string(&settings_path).unwrap_or_else(|e| {
        panic!("failed reading {settings_path}: {e}");
    });

    toml::from_str(&settings_contents).unwrap_or_else(|e| panic!("failed parsing {settings_path}: {e}"))
}
