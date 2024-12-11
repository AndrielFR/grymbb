// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the sed command handler.

use ferogram::{filter, handler, Context, Filter, Result, Router};
use grammers_client::InputMessage;

use crate::{filters, modules::i18n::I18n};

/// Setup the sed command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(filter::regex("^s/(.*)/(.*)(/(.*))?$").and(filters::sudoers()))
            .then(sed),
    )
}

/// Handles the sed command.
async fn sed(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);

    let text = ctx.text().unwrap();
    let args = text.split('/').skip(1).collect::<Vec<_>>();

    let (pattern, replacement, flags) = match args.len() {
        2 => (args[0], args[1], ""),
        3 => (args[0], args[1], args[2]),
        _ => return Ok(()),
    };

    if let Some(reply) = ctx.get_reply().await? {
        let new_text = if flags.contains('g') {
            reply.html_text().replace(pattern, replacement)
        } else {
            reply.html_text().replacen(pattern, replacement, 1)
        };

        ctx.edit_or_reply(InputMessage::html(format!(
            "<blockquote>{}</blockquote>",
            new_text
        )))
        .await?;
    } else {
        ctx.reply(InputMessage::html(t("reply_needed"))).await?;
    }

    Ok(())
}
