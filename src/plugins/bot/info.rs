// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the info command handler.

use std::time::Duration;

use ferogram::{filter, handler, Filter, Result, Router};
use grammers_client::{
    button, reply_markup,
    types::{CallbackQuery, InputMessage},
};
use maplit::hashmap;
use sysinfo::System;

use crate::{filters, modules::i18n::I18n};

/// Setup the info command.
pub fn setup() -> Router {
    Router::default()
        .handler(handler::callback_query(filter::regex("^info").and(filters::sudoers())).then(info))
}

/// Handles the info command.
async fn info(query: CallbackQuery, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    let mut info = System::new_all();

    info.refresh_cpu_usage();

    let cpu_usage = info.global_cpu_usage();
    let used_memory = info.used_memory() as f64 / 10f64.powi(9);
    let total_memory = info.total_memory() as f64 / 10f64.powi(9);
    let memory_usage = (used_memory / total_memory) * 100f64;

    let args = hashmap! {
        "os" => System::name().unwrap_or("Unknown".to_string()),
        "cpu_usage" => (cpu_usage as u64).to_string(),
        "arch" => System::cpu_arch().unwrap_or("x86_64".to_string()),
        "host" => System::host_name().unwrap_or("localhost".to_string()),
        "version" => env!("CARGO_PKG_VERSION").to_string(),
        "kernel_version" => System::kernel_version().unwrap_or("1.0.0".to_string()),
        "memory_usage" => (memory_usage as u64).to_string(),
        "used_memory" => format!("{:.2}", used_memory),
        "total_memory" => format!("{:.2}", total_memory),
    };
    query
        .answer()
        .text(t("info_updated"))
        .cache_time(Duration::from_secs(10))
        .edit(
            InputMessage::html(t_a("info_text", args)).reply_markup(&reply_markup::inline(vec![
                vec![button::inline(t("reload_button"), "info")],
            ])),
        )
        .await?;

    Ok(())
}
