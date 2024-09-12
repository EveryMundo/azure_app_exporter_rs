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

use axum::{extract::State, Json};

use crate::{global_state::GlobalState, settings::app_settings::Settings};

/// Show the exporter settings, except sensitive values
#[utoipa::path(get, tag = "Info", path = "/api/settings", responses((status = OK, body = Settings)))]
pub async fn show_settings(State(global_state): State<&GlobalState>) -> Json<&Settings> {
    Json(&global_state.settings)
}
