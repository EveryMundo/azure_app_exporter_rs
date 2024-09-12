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

//! Define a typed header to more easily detect if a request has been made from Swagger UI
//! without having to access the HTTP request's headers manually every time
//! by simply adding an extra parameter to our endpoint handler functions.
//!
//! This is used to truncate certain API responses so as not to hang the Swagger UI page when rendering the response.

use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers::{Header, HeaderMapExt};

#[derive(Debug)]
pub struct FromSwaggerUi;

// Apparently this panics if the header name is not lowercase https://github.com/hyperium/headers/issues/156#issuecomment-1826905271
static FROM_SWAGGER_UI_HEADER_NAME: HeaderName = HeaderName::from_static("from-swagger-ui");

impl Header for FromSwaggerUi {
    fn name() -> &'static HeaderName {
        &FROM_SWAGGER_UI_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        values.next().map(|_| Self).ok_or_else(axum_extra::headers::Error::invalid)
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_static("")))
    }
}

pub async fn set_swagger_ui_header<B>(swagger_ui_url: &str, mut request: axum::http::Request<B>) -> axum::http::Request<B> {
    if request
        .headers()
        .get("Referer")
        .and_then(|r| r.to_str().ok())
        .map(|r| r.trim_end_matches('/').ends_with(swagger_ui_url.trim_matches('/')))
        .unwrap_or_default()
    {
        request.headers_mut().typed_insert(FromSwaggerUi);
    }
    request
}
