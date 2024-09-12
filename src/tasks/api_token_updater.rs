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

use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::{app_metrics::TOKEN_SECONDS, global_state::GlobalState};

#[derive(Debug, Deserialize)]
struct AuthToken {
    expires_in: u64,
    access_token: String,
}

/// https://learn.microsoft.com/en-us/graph/auth-v2-service#4-request-an-access-token
pub async fn azure_api_token_updater(global_state: &GlobalState) {
    let inner = || async move {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            global_state.settings.credentials.tenant_id
        );
        tracing::debug!(url, "getting azure api token with client id and secret");

        let response: AuthToken = global_state
            .http_client
            .post(url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("scope", "https://graph.microsoft.com/.default"),
                ("client_id", &global_state.settings.credentials.client_id),
                ("client_secret", &global_state.settings.credentials.client_secret),
            ])
            .send()
            .await?
            .json()
            .await?;

        let mut azure_api_token = global_state.azure_api_token.write().expect("lock poisoned");
        *azure_api_token = response.access_token;

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(response.expires_in)
    };

    loop {
        let start = Instant::now();

        let result = inner().await;

        let elapsed = start.elapsed();
        let took_millis = elapsed.as_millis() as u64;

        let (sleep_duration, status) = match result {
            Ok(expires_in) => {
                let dur = Duration::from_secs(expires_in).mul_f64(0.9); // Sleep for 90% of the token's validity duration
                let next_update_in_millis = dur.as_millis() as u64;

                tracing::info!(took_millis, next_update_in_millis, "updated azure api token");

                (dur, "success")
            }
            Err(e) => {
                let dur = Duration::from_secs(30); // Try again in 30 seconds on error
                let next_update_in_millis = dur.as_millis() as u64;

                tracing::error!(took_millis, next_update_in_millis, error = e, "failed updating azure api token");

                (dur, "fail")
            }
        };

        metrics::histogram!(TOKEN_SECONDS, &[("status", status)]).record(elapsed);

        tokio::time::sleep(sleep_duration).await;
    }
}
