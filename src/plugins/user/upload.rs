// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the upload command handler.

use std::{io::Cursor, time::Instant};

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::{grammers_tl_types::enums::MessageEntity, InputMessage};
use maplit::hashmap;

use crate::{
    filters,
    modules::i18n::I18n,
    utils::{fetch_stream, human_readable_size},
};

/// Setup the upload command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(filters::commands(&["u", "up", "upload"]).and(filters::sudoers()))
            .then(upload),
    )
}

/// Handles the upload command.
async fn upload(ctx: Context, i18n: I18n) -> Result<()> {
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
                ctx.reply(t("reply_not_url_or_media")).await?;
                return Ok(());
            }

            ctx.edit(t("download_processing")).await?;

            let entity = url_entities[0];
            let offset = entity.offset() as usize;
            let length = entity.length() as usize;

            let url = &text[offset..(offset + length)];
            upload_file(url, ctx, &i18n).await?;
        } else {
            ctx.reply(t("reply_not_url_or_media")).await?;
        }
    } else if text.split_whitespace().count() < 2 {
        ctx.reply(t("download_not_url")).await?;
    } else {
        ctx.edit(t("download_processing")).await?;

        let url = text.split_whitespace().skip(1).next().unwrap();
        upload_file(url, ctx, &i18n).await?;
    }

    Ok(())
}

/// Uploads a file from a URL.
async fn upload_file(url: &str, ctx: Context, i18n: &I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    let time = Instant::now();
    match fetch_stream(url).await {
        Ok(stream) => {
            if stream.is_empty() {
                ctx.edit_or_reply(t("download_empty")).await?;
                return Ok(());
            }

            let file_name = stream.file_name().to_string();
            let size = stream.len();

            if size > 2 * 1024 * 1024 * 1024 {
                ctx.edit_or_reply(t("download_size_limit")).await?;
                return Ok(());
            } else if let Some(length) = stream.content_length() {
                if length != size as u64 {
                    ctx.edit_or_reply(t("download_size_mismatch")).await?;
                    return Ok(());
                }
            }

            let content_type = stream.content_type().to_string();
            ctx.edit_or_reply(InputMessage::html(t_a(
                        "upload_info",
                        hashmap! { "name" => file_name.to_string(), "type" => content_type, "size" => human_readable_size(size) },
                    )))
                    .await?;

            let mut cursor = Cursor::new(stream.as_bytes());
            let file = ctx.upload_stream(&mut cursor, size, file_name).await?;

            ctx.send(
                InputMessage::html(t_a(
                    "upload_time",
                    hashmap! { "time" => time.elapsed().as_secs_f32().to_string() },
                ))
                .document(file),
            )
            .await?;
            ctx.delete().await?;
        }
        Err(_) => {
            ctx.edit_or_reply(t("download_error")).await?;
        }
    }

    Ok(())
}
