// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the configuration module.

use std::{fs::File, io::Read};

use ferogram::Result;
use serde::{Deserialize, Serialize};

const PATH: &str = "./assets/config.toml";

/// Configuration.
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub telegram: Telegram,
    pub bot: Bot,
    pub user: User,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut file = File::open(PATH)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Telegram configuration.
#[derive(Deserialize, Serialize)]
pub struct Telegram {
    pub api_id: i32,
    pub api_hash: String,
    pub flood_sleep_threshold: u32,
}

/// Bot configuration.
#[derive(Deserialize, Serialize)]
pub struct Bot {
    pub token: String,
    pub catch_up: bool,
    pub session_file: String,
}

/// User configuration.
#[derive(Deserialize, Serialize)]
pub struct User {
    pub phone_number: String,
    pub catch_up: bool,
    pub session_file: String,
}
