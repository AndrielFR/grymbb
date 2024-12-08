// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the purge command handler.

use std::time::Duration;

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::types::InputMessage;
use maplit::hashmap;

use crate::{filters, modules::i18n::I18n};

/// Setup the purge command.
pub fn setup() -> Router {
    Router::default()
        .handler(
            handler::new_message(filters::command("purge").and(filters::sudoers())).then(purge),
        )
        .handler(
            handler::new_message(filters::command("purgeme").and(filters::sudoers()))
                .then(purge_me),
        )
}

/// Handles the purge command.
async fn purge(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    if let Some(reply) = ctx.get_reply().await? {
        let msg = ctx.message().await.unwrap();
        let message_ids = (reply.id()..=(msg.id() - 1)).collect::<Vec<_>>();
        let total_messages = message_ids.len();
        let mut purged_messages = 0;

        ctx.edit(InputMessage::html(t_a(
            "purging",
            hashmap! {
                "count" => total_messages.to_string(),
            },
        )))
        .await?;

        let mut waited = 0;
        for chunk in message_ids.chunks(100) {
            match ctx.delete_messages(chunk.to_vec()).await {
                Ok(count) => purged_messages += count,
                Err(e) if e.is("MESSAGE_ID_INVALID") => continue,
                Err(e) if e.is("MESSAGE_DELETE_FORBIDDEN") => {
                    ctx.edit(t("you_dont_have_perms")).await?;

                    return Ok(());
                }
                Err(e) if e.is("FLOOD_WAIT") => {
                    let time = 5 * (waited + 1);
                    waited += 1;

                    let sent = ctx
                        .reply(InputMessage::html(t_a(
                            "flood_wait",
                            hashmap! { "seconds" => time.to_string() },
                        )))
                        .await?;

                    tokio::time::sleep(Duration::from_secs(time)).await;
                    sent.delete().await?;
                }
                Err(e) => {
                    log::error!("failed to purge messages: {}", e);
                    ctx.edit(t("purge_error")).await?;

                    return Ok(());
                }
            };
        }

        ctx.edit(InputMessage::html(t_a(
            "purged",
            hashmap! {
                "count" => purged_messages.to_string(),
            },
        )))
        .await?;

        tokio::time::sleep(Duration::from_secs(4)).await;
        ctx.delete().await?;
    } else {
        let sent = ctx.reply(InputMessage::html(t("reply_needed"))).await?;

        tokio::time::sleep(Duration::from_secs(4)).await;
        sent.delete().await?;
        ctx.delete().await?;
    }

    Ok(())
}

/// Handles the purgeme command.
async fn purge_me(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    if let Some(reply) = ctx.get_reply().await? {
        let msg = ctx.message().await.unwrap();
        let sender = msg.sender().expect("Message has no sender");
        let message_ids = (reply.id()..=(msg.id() - 1)).collect::<Vec<_>>();
        let mut purged_messages = 0;

        ctx.edit(InputMessage::html(t("purging_me"))).await?;

        let mut waited = 0;
        for message_id in message_ids {
            match ctx.get_message(message_id).await {
                Ok(Some(msg)) => {
                    if let Some(snd) = msg.sender() {
                        if snd.id() == sender.id() {
                            purged_messages += 1;
                            msg.delete().await?;
                        }
                    }
                }
                Err(e) if e.is("FLOOD_WAIT") => {
                    let time = 5 * (waited + 1);
                    waited += 1;

                    let sent = ctx
                        .reply(InputMessage::html(t_a(
                            "flood_wait",
                            hashmap! { "seconds" => time.to_string() },
                        )))
                        .await?;

                    tokio::time::sleep(Duration::from_secs(time)).await;
                    sent.delete().await?;
                }
                Err(e) => {
                    log::error!("failed to get message: {}", e);
                    ctx.edit(InputMessage::html(t("purge_error"))).await?;

                    return Ok(());
                }
                _ => continue,
            }
        }

        ctx.edit(InputMessage::html(t_a(
            "purged_me",
            hashmap! {
                "count" => purged_messages.to_string(),
            },
        )))
        .await?;

        tokio::time::sleep(Duration::from_secs(4)).await;
        ctx.delete().await?;
    } else {
        let sent = ctx.reply(InputMessage::html(t("reply_needed"))).await?;

        tokio::time::sleep(Duration::from_secs(4)).await;
        sent.delete().await?;
        ctx.delete().await?;
    }

    Ok(())
}
