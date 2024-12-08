// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains some custom filters.

use std::sync::Arc;

use ferogram::{filter, Filter};
use grammers_client::{types::inline, Update};

const SUDOER_LIST: [i64; 1] = [1155717290];

/// Custom filter that checks if the user is a sudoer.
pub fn sudoers() -> impl Filter {
    filter::me.or(Arc::new(move |_client, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                if let Some(sender) = message.sender() {
                    SUDOER_LIST.contains(&sender.id())
                } else {
                    false
                }
            }
            Update::CallbackQuery(query) => {
                let sender = query.sender();
                let value = SUDOER_LIST.contains(&sender.id());

                if !value {
                    query
                        .answer()
                        .alert("You are not allowed to do that.")
                        .send()
                        .await
                        .expect("Failed to send alert message.");
                }

                value
            }
            Update::InlineQuery(query) => {
                let sender = query.sender();
                let value = SUDOER_LIST.contains(&sender.id());

                if !value {
                    query
                        .answer(vec![inline::query::Article::new(
                            "You are not allowed to do that.",
                            "You are not allowed to do that.",
                        )
                        .into()])
                        .send()
                        .await
                        .expect("Failed to send article.");
                }

                value
            }
            _ => false,
        }
    }))
}

/// Custom `command` filter with prefixes to user instance.
pub fn command(pat: &'static str) -> impl Filter {
    filter::command_with(&[";", ",", "."], pat)
}

/// Custom `commands` filter with prefixes to user instance.
pub fn commands(pats: &'static [&'static str]) -> impl Filter {
    filter::commands_with(&[";", ",", "."], pats)
}
