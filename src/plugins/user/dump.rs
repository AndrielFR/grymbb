// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the dump command handler.

use std::io::Cursor;

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::InputMessage;

use crate::{filters, Dump};

/// Setup the dump command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(filters::commands(&["du", "dump"]).and(filters::sudoers())).then(dump),
    )
}

/// Handles the dump command.
async fn dump(ctx: Context) -> Result<()> {
    if let Some(reply) = ctx.get_reply().await? {
        let json = reply.dump();

        match ctx
            .edit_or_reply(InputMessage::html(format!(
                "<blockquote>{}</blockquote>",
                json
            )))
            .await
        {
            Err(e) if e.is("MESSAGE_TOO_LONG") => {
                let bytes = json.as_bytes();
                let size = bytes.len();

                let mut stream = Cursor::new(bytes);
                let file = ctx
                    .upload_stream(&mut stream, size, "reply_dump.json".to_string())
                    .await?;

                ctx.send(InputMessage::text("").document(file)).await?;
            }
            _ => {}
        }
    }

    let msg = ctx.message().await.unwrap();
    let json = msg.dump();

    match ctx
        .edit_or_reply(InputMessage::html(format!(
            "<blockquote>{}</blockquote>",
            json
        )))
        .await
    {
        Err(e) if e.is("MESSAGE_TOO_LONG") => {
            let bytes = json.as_bytes();
            let size = bytes.len();

            let mut stream = Cursor::new(bytes);
            let file = ctx
                .upload_stream(&mut stream, size, "dump.json".to_string())
                .await?;

            ctx.send(InputMessage::text("").document(file)).await?;
            ctx.delete().await?;
        }
        _ => {}
    }

    Ok(())
}
