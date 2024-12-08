// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the eval command handler.

use std::{
    io::{Cursor, Read},
    process::{Command, Stdio},
    time::Instant,
};

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::InputMessage;
use maplit::hashmap;

use crate::{filters, modules::i18n::I18n};

/// Setup the eval command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(filters::commands(&["e", "eval", "exec"]).and(filters::sudoers()))
            .then(eval),
    )
}

/// Handles the eval command.
async fn eval(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    if let Some(text) = ctx.text() {
        let input = text
            .trim()
            .split_whitespace()
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ");

        ctx.edit(InputMessage::html(t_a(
            "evaluating",
            hashmap! { "input" => input.clone() },
        )))
        .await?;
        let time = Instant::now();

        if let Ok(mut child) = Command::new("rust-script")
            .args(["-e", &input])
            .env("RUST_LOG", "off")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            if let Ok(status) = child.wait() {
                let elapsed = time.elapsed().as_secs_f64();

                let mut buf = String::new();

                if status.success() {
                    let mut stdout = child.stdout.take().unwrap();
                    stdout.read_to_string(&mut buf)?;
                } else {
                    let mut stderr = child.stderr.take().unwrap();
                    stderr.read_to_string(&mut buf)?;
                }

                let output = buf.trim_ascii().to_string();
                if output.len() > 4000 {
                    let bytes = output.as_bytes();
                    let size = bytes.len();

                    let mut cursor = Cursor::new(bytes);
                    let file = ctx
                        .client()
                        .upload_stream(&mut cursor, size, "output.txt".to_string())
                        .await?;

                    ctx.edit(InputMessage::html(t_a(
                        "eval_input",
                        hashmap! { "input" => input, "time" => elapsed.to_string() },
                    )))
                    .await?;
                    ctx.reply(InputMessage::html(t("eval_output_file")).document(file))
                        .await?;

                    return Ok(());
                }

                ctx.edit(InputMessage::html(t_a(
                "eval_output",
                hashmap! { "input" => input, "output" => output, "time" => elapsed.to_string() },
                )))
                .await?;
            } else {
                ctx.reply(t("eval_failure")).await?;
            }
        } else {
            ctx.reply(t("eval_failure")).await?;
        }
    } else {
        ctx.reply(t("eval_no_code")).await?;
    }

    Ok(())
}
