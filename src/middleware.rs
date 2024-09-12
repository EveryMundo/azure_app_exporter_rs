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

use std::time::Instant;

use axum::{
    body::HttpBody,
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use metrics::{counter, histogram};

use crate::app_metrics::{REQUESTS_TOTAL, REQUEST_SECONDS, REQUEST_SIZE, RESPONSE_SIZE};

pub async fn logging(request: Request, next: Next) -> Response {
    let matched_path = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_string()
    } else {
        "unsupported-url".into()
    };

    let path = request.uri().path().to_string();
    let host = request.uri().authority().map(|authority| authority.to_string()).unwrap_or_default();
    let method = request.method().to_string();

    let request_bytes = request.size_hint().exact().unwrap_or_default() as f64;

    let start = Instant::now();

    let response = next.run(request).await;

    let latency = start.elapsed();

    let status = response.status().as_u16();

    let response_bytes = response.size_hint().exact().unwrap_or_default() as f64;

    tracing::info!(
        method,
        host,
        path,
        status,
        request_bytes,
        response_bytes,
        // HACK cast to u64 because, as of writing, `tracing` formats u128 as a string instead of a number when logging with json lines
        latency_ms = latency.as_millis() as u64,
        "handled request"
    );

    let labels = [("method", method), ("host", host), ("path", matched_path), ("status", status.to_string())];

    counter!(REQUESTS_TOTAL, &labels).increment(1);
    histogram!(REQUEST_SECONDS, &labels).record(latency);
    histogram!(REQUEST_SIZE, &labels).record(request_bytes);
    histogram!(RESPONSE_SIZE, &labels).record(response_bytes);

    response
}
