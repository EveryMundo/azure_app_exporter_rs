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

//! Gather additional information like the Rust compiler version and make it available to our main code at compile time
//! as environment variables.

fn main() {
    let rustc_output = String::from_utf8(
        std::process::Command::new("rustc")
            .arg("-vV")
            .output()
            .expect("failed running rustc")
            .stdout,
    )
    .expect("failed converting rustc output to String");

    let rust_info = rustc_output
        .lines()
        .skip(2)
        .map(|s| s.split_once(": ").expect("split failed"))
        .collect::<Vec<(_, _)>>();

    let labels = rust_info.iter().map(|(label, _)| label).collect::<Vec<_>>();
    for label in ["commit-hash", "commit-date", "host", "release", "LLVM version"] {
        if !labels.contains(&&label) {
            panic!("label {label} not in output of rustc -vV");
        }
    }

    let jsonified = serde_json::to_string(&rust_info).expect("failed serializing rust info");

    println!("cargo:rustc-env=RUST_INFO={jsonified}");
}
