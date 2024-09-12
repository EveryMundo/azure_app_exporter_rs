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

//! Declarations for all metrics used throughout the project, their `# HELP` descriptions
//! and usage for one-time metrics like app_info.

use std::time::Duration;

use metrics::{counter, describe_counter, describe_gauge, describe_histogram};

pub const REQUESTS_TOTAL: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "requests_total");
pub const REQUEST_SECONDS: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "request_duration_seconds");
pub const REQUEST_SIZE: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "request_size_bytes");
pub const RESPONSE_SIZE: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "response_size_bytes");

pub const TOKEN_SECONDS: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "azure_api_token_update_duration_seconds");
pub const APPLICATIONS_SECONDS: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "azure_applications_update_duration_seconds");

pub const APPLICATION_PASSWORD_SECONDS: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "azure_application_password_remaining_seconds");

const APP_INFO: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "app_info");
const RUST_INFO: &str = concat!(env!("CARGO_CRATE_NAME"), "_", "rust_info");

pub fn setup_metrics() {
    describe_counter!(APP_INFO, "Information about the application.");
    describe_counter!(RUST_INFO, "Information about the Rust environment.");

    describe_counter!(
        REQUESTS_TOTAL,
        "Number of HTTP requests processed, partitioned by HTTP method, host, url and status code."
    );
    describe_histogram!(REQUEST_SECONDS, "The HTTP request latencies in seconds.");
    describe_histogram!(REQUEST_SIZE, "The HTTP request sizes in bytes.");
    describe_histogram!(RESPONSE_SIZE, "The HTTP response sizes in bytes.");

    describe_histogram!(TOKEN_SECONDS, "How many seconds it takes to update the Azure API token.");

    describe_histogram!(
        APPLICATIONS_SECONDS,
        "How many seconds it takes to update the in-memory cache of Azure applications."
    );

    describe_gauge!(APPLICATION_PASSWORD_SECONDS, "Seconds remaining until the password credential expires.");

    counter!(APP_INFO, &[("version", env!("CARGO_PKG_VERSION"))]).increment(1);

    let rust_info: Vec<(String, String)> = serde_json::from_str(env!("RUST_INFO")).expect("failed deserializing RUST_INFO env var");
    counter!(RUST_INFO, &rust_info).increment(1);

    let process_collector = metrics_process::Collector::default();
    process_collector.describe();

    // Periodically update the "process_" metrics
    tokio::spawn(async move {
        loop {
            process_collector.collect();
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}
