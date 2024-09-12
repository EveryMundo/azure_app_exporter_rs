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

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{response::ErasedJson, TypedHeader};

use crate::{global_state::GlobalState, utils::FromSwaggerUi};

/// Show all Azure applications cached in the exporter (truncated in Swagger UI to 50 entries)
///
/// Call this endpoint outside Swagger UI to see full response
#[utoipa::path(get, tag = "Applications", path = "/api/apps", responses((status = OK, body = HashMap<String, AzureApplication>)))]
pub async fn get_all_applications(State(global_state): State<&GlobalState>, from_swagger: Option<TypedHeader<FromSwaggerUi>>) -> ErasedJson {
    let applications = global_state.applications.read().expect("lock poisoned");
    if from_swagger.is_some() {
        ErasedJson::new(applications.iter().take(50).collect::<HashMap<_, _>>())
    } else {
        ErasedJson::new(&*applications)
    }
}

/// Show Azure application by ID
#[utoipa::path(get, tag = "Applications", path = "/api/apps/{id}",
    params(("id" = String, Path, description = "ID of Azure application to lookup")),
    responses((status = OK, body = AzureApplication), (status = NOT_FOUND, description = "No application found by the given ID"))
)]
pub async fn get_application_by_id(State(global_state): State<&GlobalState>, Path(id): Path<String>) -> Result<ErasedJson, StatusCode> {
    if let Some(app) = global_state.applications.read().expect("lock poisoned").get(&id) {
        Ok(ErasedJson::new(app))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
