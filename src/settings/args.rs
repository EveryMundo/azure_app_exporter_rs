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

const HELP: &str = concat!(
    env!("CARGO_CRATE_NAME"),
    " v",
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("CARGO_PKG_DESCRIPTION"),
    r#"

Usage:
  show-settings-example
    Show example settings file and exit
    Path to settings file is given on the environment variable AZURE_APP_EXPORTER_SETTINGS_PATH
    Default settings path is /etc/azure_app_exporter/settings.toml
"#
);

pub fn check_args() {
    let args: Vec<_> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{HELP}");

        std::process::exit(0);
    } else if args.iter().any(|a| a == "show-settings-example") {
        print!("{}", include_str!("../../settings_example.toml"));

        std::process::exit(0);
    }
}
