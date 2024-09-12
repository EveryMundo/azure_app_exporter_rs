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

use crate::{app_metrics::APPLICATIONS_SECONDS, global_state::GlobalState, types::applications::AzureApplications};

/// https://learn.microsoft.com/en-us/graph/query-parameters
/// https://learn.microsoft.com/en-us/graph/api/application-list?view=graph-rest-1.0
pub async fn azure_applications_updater(global_state: &GlobalState) {
    // This fn is spawned in a thread simultaneously with another thread
    // responsible for updating the api token, so we should wait for it to finish
    while global_state.azure_api_token.read().expect("lock poisoned").is_empty() {
        tracing::warn!("azure api token not yet acquired, sleeping 5 seconds");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let get_applications = |url| async move {
        tracing::debug!(url, "getting azure applications with api token");

        global_state
            .http_client
            .get(url)
            .bearer_auth(global_state.azure_api_token.read().expect("lock poisoned"))
            .send()
            .await?
            .json::<AzureApplications>()
            .await
    };

    let inner = || async move {
        let mut response = get_applications(format!(
            "{}?$top={}&$select=id,appId,displayName,createdDateTime,passwordCredentials",
            global_state.settings.applications.url, global_state.settings.applications.results_per_page
        ))
        .await?;

        while let Some(next_link) = response.next_link {
            let mut next_response = get_applications(next_link).await?;

            response.next_link = next_response.next_link;
            response.value.append(&mut next_response.value);
        }

        let parsed_applications = response.value.into_iter().map(|application| (application.id.clone(), application));

        let mut applications = global_state.applications.write().expect("lock poisoned");
        applications.clear();
        applications.extend(parsed_applications);

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    };

    loop {
        let start = Instant::now();

        let result = inner().await;

        let elapsed = start.elapsed();
        let took_millis = elapsed.as_millis() as u64;
        let next_update_in_millis = global_state.settings.applications.cache_refresh_interval.as_millis() as u64;

        let applications_cached = global_state.applications.read().expect("lock poisoned").len();

        let status_label = match result {
            Ok(_) => {
                tracing::info!(took_millis, next_update_in_millis, applications_cached, "updated azure applications");

                "success"
            }
            Err(e) => {
                tracing::error!(
                    took_millis,
                    next_update_in_millis,
                    applications_cached,
                    error = e,
                    "failed updating azure applications"
                );

                "fail"
            }
        };

        metrics::histogram!(APPLICATIONS_SECONDS, &[("status", status_label)]).record(elapsed);

        tokio::time::sleep(global_state.settings.applications.cache_refresh_interval).await
    }
}
