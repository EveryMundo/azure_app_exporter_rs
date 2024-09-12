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

use std::{path::Path, sync::Arc};

use axum::{response::Redirect, routing::get, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use metrics_exporter_prometheus::Matcher;
use metrics_util::MetricKindMask;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use azure_app_exporter::{
    app_metrics,
    global_state::GlobalState,
    middleware, routes,
    settings::{app_settings, args},
    tasks, types, utils,
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Azure app exporter", contact()),
    paths(routes::metrics, routes::show_settings, routes::get_all_applications, routes::get_application_by_id),
    components(schemas(app_settings::Settings, types::applications::AzureApplication))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    args::check_args();

    // Setup json-line logging. Change log level and targets with `RUST_LOG=crate1=trace,crate2=info`
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .json()
        .flatten_event(true)
        .with_current_span(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .init();

    // We leak this because it needs to live as long as the application, be shared between threads
    // and because it's much easier to implicitly Copy a reference than explicitly Clone an Arc
    let global_state = &*Box::leak(Box::new(GlobalState::new()));

    if global_state.settings.debug.no_verify_tls {
        tracing::warn!("flag no_verify_tls is enabled, CERTIFICATES ON FOREIGN API REQUESTS WILL NOT BE VALIDATED!")
    }

    let metric_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        // Remove gauge metrics that have not been updated for the given span of time
        .idle_timeout(MetricKindMask::GAUGE, global_state.settings.metrics.prune_interval)
        .set_buckets_for_metric(
            // Required to use real Prometheus histograms over summaries
            Matcher::Suffix("_duration_seconds".into()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .expect("failed setting buckets")
        .set_buckets_for_metric(
            Matcher::Suffix("_size_bytes".into()),
            &[
                512.0,       // 512 B
                1024.0,      // 1,024 B
                2048.0,      // 2.0 KiB
                5120.0,      // 5.0 KiB
                10240.0,     // 10.0 KiB
                25600.0,     // 25.0 KiB
                51200.0,     // 50.0 KiB
                102400.0,    // 100.0 KiB
                256000.0,    // 250.0 KiB
                512000.0,    // 500.0 KiB
                1048576.0,   // 1024.0 KiB
                2097152.0,   // 2.0 MiB
                5242880.0,   // 5.0 MiB
                10485760.0,  // 10.0 MiB
                26214400.0,  // 25.0 MiB
                52428800.0,  // 50.0 MiB
                104857600.0, // 100.0 MiB
            ],
        )
        .expect("failed setting buckets")
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    app_metrics::setup_metrics();

    let router = if global_state.settings.openapi.enabled {
        Router::new()
            .merge(
                SwaggerUi::new(&global_state.settings.openapi.swagger_ui_url)
                    .url(global_state.settings.openapi.docs_url.clone(), ApiDoc::openapi())
                    .config(Config::default().use_base_layout().display_request_duration(true)),
            )
            .route("/", get(|| async { Redirect::to(&global_state.settings.openapi.swagger_ui_url) }))
    } else {
        Router::new()
    }
    .route("/metrics", get(routes::metrics))
    .route("/api/settings", get(routes::show_settings))
    .route("/api/apps", get(routes::get_all_applications))
    .route("/api/apps/:id", get(routes::get_application_by_id))
    .with_state(global_state)
    .layer(Extension(metric_handle))
    .layer(axum::middleware::map_request(|request| {
        utils::set_swagger_ui_header(&global_state.settings.openapi.swagger_ui_url, request)
    }))
    .layer(axum::middleware::from_fn(middleware::logging));

    tracing::info!("beginning to serve on {}", global_state.settings.web.listen_address);
    tracing::info!("metrics endpoint: {}/metrics", global_state.settings.web.listen_address);
    tracing::info!(
        "swagger endpoint: {}{}",
        global_state.settings.web.listen_address,
        global_state.settings.openapi.swagger_ui_url
    );

    if global_state.settings.applications.enabled {
        tokio::spawn(tasks::azure_api_token_updater(global_state));
        tokio::spawn(tasks::azure_applications_updater(global_state));
        tokio::spawn(tasks::azure_metrics_updater(global_state));
    }

    if let (Some(cert_path), Some(key_path)) = (&global_state.settings.web.cert_file, &global_state.settings.web.key_file) {
        let tls_config = build_tls_config(cert_path, key_path, &global_state.settings.tls);

        axum_server::bind_rustls(global_state.settings.web.listen_address, tls_config)
            .serve(router.into_make_service())
            .await
            .expect("failed starting server");
    } else {
        tracing::warn!("no cert or key file provided in settings.toml, running server in HTTP mode");
        axum_server::bind(global_state.settings.web.listen_address)
            .serve(router.into_make_service())
            .await
            .expect("failed starting server");
    }
}

// If we want to select which TLS ciphers and protocols we want to use, we'll have to build the TLS config a bit more manually
fn build_tls_config(cert_path: &Path, key_path: &Path, tls_settings: &app_settings::Tls) -> RustlsConfig {
    let mut cert_reader = std::io::BufReader::new(std::fs::File::open(cert_path).expect("tls cert path must be a file"));
    let mut key_reader = std::io::BufReader::new(std::fs::File::open(key_path).expect("tls key path must be a file"));

    let tls_certs = rustls_pemfile::certs(&mut cert_reader)
        .flatten()
        .map(|c| rustls::Certificate(c.to_vec()))
        .collect::<Vec<_>>();

    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .flatten()
        .map(|k| rustls::PrivateKey(k.secret_pkcs8_der().to_vec()))
        .next()
        .expect("provided tls key must be valid");

    let mut server_config = rustls::server::ServerConfig::builder()
        .with_cipher_suites(&tls_settings.rustls_cipher_suites())
        .with_kx_groups(&tls_settings.rustls_kx_groups())
        .with_protocol_versions(&tls_settings.rustls_protocol_versions())
        .expect("tls config must be valid. If this fails, perhaps an invalid cipher suite and protocol version combo are configured")
        .with_no_client_auth()
        .with_single_cert(tls_certs, tls_key)
        .expect("tls config must be valid");

    // We have to set this ourselves since we're building the [`ServerConfig`] from scratch
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    RustlsConfig::from_config(Arc::new(server_config))
}
