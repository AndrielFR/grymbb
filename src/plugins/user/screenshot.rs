// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the screenshot command handler.

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::{grammers_tl_types::enums::MessageEntity, InputMessage};

use crate::{filters, modules::i18n::I18n, utils::take_a_screenshot};

/// Setup the screenshot command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(
            filters::commands(&["ss", "screenshot", "pp", "print"]).and(filters::sudoers()),
        )
        .then(screenshot),
    )
}

/// Handles the screenshot command.
async fn screenshot(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);

    let text = ctx.text().unwrap();
    if let Some(reply) = ctx.get_reply().await? {
        let text = reply.text().to_string();

        if let Some(entities) = reply.fmt_entities() {
            let url_entities = entities
                .into_iter()
                .filter(|entity| {
                    matches!(entity, MessageEntity::Url(_) | MessageEntity::TextUrl(_))
                })
                .collect::<Vec<_>>();

            if url_entities.is_empty() {
                ctx.reply(t("reply_not_url")).await?;
                return Ok(());
            }

            let msg = ctx.edit_or_reply(t("screenshot_processing")).await?;

            let entity = url_entities[0];
            let offset = entity.offset() as usize;
            let length = entity.length() as usize;

            let url = &text[offset..(offset + length)];
            match take_a_screenshot(url.to_string()).await {
                Ok(photo_url) => {
                    ctx.send(InputMessage::html("").photo_url(photo_url))
                        .await?;
                    ctx.delete().await?;
                }
                Err(_) => {
                    msg.edit(t("screenshot_error")).await?;
                }
            }
        } else {
            ctx.reply(t("reply_not_url")).await?;
        }
    } else if text.split_whitespace().count() < 2 {
        ctx.reply(t("screenshot_no_url")).await?;
    } else if text.split_whitespace().count() > 2 {
        ctx.reply(t("screenshot_many_urls")).await?;
    } else {
        let msg = ctx.edit_or_reply(t("screenshot_processing")).await?;

        let url = text.split_whitespace().skip(1).next().unwrap();
        match take_a_screenshot(url.to_string()).await {
            Ok(photo_url) => {
                ctx.send(InputMessage::text(url).photo_url(photo_url))
                    .await?;
                ctx.delete().await?;
            }
            Err(_) => {
                msg.edit(t("screenshot_error")).await?;
            }
        }
    }

    Ok(())
}
