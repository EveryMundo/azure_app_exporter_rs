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

use std::{collections::HashMap, sync::RwLock, time::Duration};

use crate::{
    settings::app_settings::{self, Settings},
    types::applications::AzureApplication,
};

/// Struct containing all the data we want to easily access and mutate throughout the project.
///
/// This struct is usually leaked to get a 'static ref to it which we can pass around threads and functions
/// easily without explicitly cloning it. This is memory safe since we only instantiate the struct once
/// at the start of the program and let it live as long as the program lives.
///
/// Once the program terminates the struct isn't freed since we leaked it,
/// but any self-respecting OS automatically reclaims unfreed memory after a process terminates so all is good.
pub struct GlobalState {
    pub settings: Settings,
    pub http_client: reqwest::Client,
    /// HashMap of id -> application
    pub applications: RwLock<HashMap<String, AzureApplication>>,
    pub azure_api_token: RwLock<String>,
}

impl GlobalState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let settings = app_settings::parse();

        let http_client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(settings.debug.no_verify_tls)
            .timeout(Duration::from_secs(60 * 2))
            .user_agent(concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("must create http client");

        Self {
            settings,
            http_client,
            applications: RwLock::default(),
            azure_api_token: RwLock::default(),
        }
    }
}
