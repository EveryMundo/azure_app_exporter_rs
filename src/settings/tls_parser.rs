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

//! We need to be able to serialize and deserialize the options in the [tls] section of the settings file and convert them to the appropriate [`rustls`] types.
//! However the [`rustls`] types like [`rustls::SupportedCipherSuite`] do not impl serde and they are foreign so we cannot impl it for them.
//! Serde does support a `remote` option, but it requires making a copy of the foreign data type which is a deal-breaker when that type is complex and contains more nested types.
//! Consequently, it's much easier to declare our own enums, impl [`From`] to describe how to convert them to the proper [`rustls`] types,
//! put them in the [`super::app_settings::Tls`] struct, and simply convert them to their equivalent [`rustls`] type when needed.
//!
//! It's also possible to use the [`rustls`] types on the [`super::app_settings::Tls`] struct directly and use `serde(with = "...")`
//! to tell serde how to serialize and deserialize them using our types as an intermediary. However, that would require even more code than what we have now
//! because we would also need to impl [`From<rustls::SupportedCipherSuite>`] for [`CipherSuite`] and so on. It's not worth it.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum CipherSuite {
    TLS13_AES_256_GCM_SHA384,
    TLS13_AES_128_GCM_SHA256,
    TLS13_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
}

impl From<CipherSuite> for rustls::SupportedCipherSuite {
    fn from(value: CipherSuite) -> Self {
        use rustls::cipher_suite::*;
        match value {
            CipherSuite::TLS13_AES_256_GCM_SHA384 => TLS13_AES_256_GCM_SHA384,
            CipherSuite::TLS13_AES_128_GCM_SHA256 => TLS13_AES_128_GCM_SHA256,
            CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => TLS13_CHACHA20_POLY1305_SHA256,
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum KxGroup {
    X25519,
    SECP256R1,
    SECP384R1,
}

impl From<KxGroup> for &'static rustls::SupportedKxGroup {
    fn from(value: KxGroup) -> Self {
        use rustls::kx_group::*;
        match value {
            KxGroup::X25519 => &X25519,
            KxGroup::SECP256R1 => &SECP256R1,
            KxGroup::SECP384R1 => &SECP384R1,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum ProtocolVersion {
    TLS13,
    TLS12,
}

impl From<ProtocolVersion> for &'static rustls::SupportedProtocolVersion {
    fn from(value: ProtocolVersion) -> Self {
        use rustls::version::*;
        match value {
            ProtocolVersion::TLS13 => &TLS13,
            ProtocolVersion::TLS12 => &TLS12,
        }
    }
}
