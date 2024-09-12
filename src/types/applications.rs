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

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

/// https://learn.microsoft.com/en-us/graph/api/resources/application?view=graph-rest-1.0#properties
#[derive(Debug, Deserialize)]
pub struct AzureApplications {
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<AzureApplication>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AzureApplication {
    pub id: String,
    pub app_id: String,
    pub display_name: Option<String>,
    #[schema(inline)]
    pub password_credentials: Vec<PasswordCredential>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasswordCredential {
    pub key_id: String,
    pub display_name: Option<String>,
    #[serde(deserialize_with = "parse_date_time")]
    pub end_date_time: Option<DateTime<Utc>>,
}

impl PasswordCredential {
    /// Return the remaining seconds until the password credential expires
    /// If an end time is not set, return positive infinity
    pub fn remaining_seconds(&self) -> f64 {
        let Some(ref end_date_time) = self.end_date_time else {
            return f64::INFINITY;
        };

        (*end_date_time - Utc::now()).num_seconds() as f64
    }
}

fn parse_date_time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error> {
    let maybe_string_time = Option::<String>::deserialize(deserializer)?;

    // Format specifiers at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    maybe_string_time
        .map(|string_time| {
            NaiveDateTime::parse_from_str(&string_time, "%+")
                .map(|time| time.and_utc())
                .map_err(|e| serde::de::Error::custom(format!("invalid time '{string_time}', expected format ISO 8601 or RFC 3339: {e}")))
        })
        .transpose()
}
