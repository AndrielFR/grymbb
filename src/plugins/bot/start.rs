// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the start command handler.

use ferogram::{filter, handler, Context, Filter, Result, Router};

use crate::{filters, modules::i18n::I18n};

/// Setup the start command.
pub fn setup() -> Router {
    Router::default()
        .handler(handler::new_message(filter::command("start").and(filters::sudoers())).then(start))
}

/// Handles the start command.
async fn start(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);

    ctx.reply(t("start_text")).await?;

    Ok(())
}
