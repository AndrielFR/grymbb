// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This is the main module of the bot.

use std::{ops::ControlFlow, time::Duration};

use config::Config;
use ferogram::{Client, Context, Injector, Result};
use grammers_client::{
    types::{self, inline},
    ReconnectionPolicy,
};
use modules::i18n::I18n;
use tokio::sync::mpsc;

mod config;
mod filters;
mod modules;
mod plugins;

/// The receiver of the channel.
pub type Receiver = mpsc::Receiver<crate::Message>;

/// The sender of the channel.
pub type Sender = mpsc::Sender<crate::Message>;

/// A custom reconnection policy.
struct MyPolicy;

impl ReconnectionPolicy for MyPolicy {
    fn should_retry(&self, attempt: usize) -> ControlFlow<(), Duration> {
        let max_attempts = 5;

        if attempt >= max_attempts {
            log::error!("Max attempts reached, stopping reconnection policy");

            ControlFlow::Break(())
        } else {
            let time = 5 * attempt;
            log::warn!("Failed to reconnect, retrying in {} seconds", time);

            ControlFlow::Continue(Duration::from_secs(time as u64))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Set the log level to info if it is not set.
    if let Err(_) = std::env::var("RUST_LOG") {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    // Initialize the logger.
    env_logger::init();

    // Load the configuration.
    let config = Config::load()?;

    // Set shared values.
    let api_id = config.telegram.api_id;
    let api_hash = &config.telegram.api_hash;
    let app_version = "1.0.0";
    let lang_code = "pt";
    let flood_sleep_threshold = config.telegram.flood_sleep_threshold;

    // Construct and connect bot instance.
    let mut bot =
        Client::bot(config.bot.token)
            .api_id(api_id)
            .api_hash(api_hash)
            .session_file(config.bot.session_file)
            .app_version(app_version)
            .lang_code(lang_code)
            .catch_up(config.bot.catch_up)
            .flood_sleep_threshold(flood_sleep_threshold)
            .reconnection_policy(&MyPolicy)
            .on_err(|_, _, err| async move {
                log::error!("An error occurred whitin bot instance: {}", err)
            })
            .build_and_connect()
            .await?;

    // Construct and connect user instance.
    let mut user = Client::user(config.user.phone_number)
        .api_id(api_id)
        .api_hash(api_hash)
        .session_file(config.user.session_file)
        .app_version(app_version)
        .lang_code(lang_code)
        .catch_up(config.user.catch_up)
        .flood_sleep_threshold(flood_sleep_threshold)
        .reconnection_policy(&MyPolicy)
        .on_err(|_, _, err| async move {
            log::error!("An error occurred whitin user instance: {}", err)
        })
        .build_and_connect()
        .await?;

    // Construct the i18n module and load it.
    let mut i18n = I18n::with(lang_code);
    i18n.load();

    // Create a dependency injector.
    let mut injector = Injector::default();

    // Inject the i18n module into the injector.
    injector.insert(i18n);

    // Create a channel to communicate between the clients.
    let (tx, rx) = mpsc::channel::<Message>(10);

    // Inject the channel's sender into the injector.
    injector.insert(tx);

    // Clone the bot and user inner instances to be used inside the plugins.
    let bot_inner = bot.inner().clone();
    let user_inner = user.inner().clone();

    // Register the dispatcher of each client.
    bot = bot.dispatcher(|_| plugins::bot(user_inner, injector.clone()));
    user = user.dispatcher(|_| plugins::user(bot_inner, injector));

    // Clone the bot and user instances to be used inside the task.
    let bot_inner = bot.inner().clone();
    let user_inner = user.inner().clone();

    // Creates a new bot's context.
    let bot_ctx = bot.new_ctx();

    // Spawn a task to handle the messages.
    tokio::task::spawn(async move {
        handle_message(bot_inner, user_inner, rx, bot_ctx)
            .await
            .expect("Failed to handle message between the clients");
    });

    // Start the clients.
    bot.run().await?;
    user.run().await?;

    // Wait for Ctrl+C to stop the clients.
    ferogram::wait_for_ctrl_c().await;

    Ok(())
}

/// The action to be taken with the message.
#[derive(Default)]
pub enum Action {
    /// Sends a message.
    SendMessage(types::Chat, types::InputMessage),
    /// Sends a via bot message.
    SendViaBotMessage(types::Chat, types::InputMessage),
    /// Edits a message.
    EditMessage(types::Chat, i32, types::InputMessage),
    /// Undefined action.
    #[default]
    Undefined,
}

/// The type of the message.
#[derive(PartialEq)]
pub enum Recipient {
    /// A message from the user to the bot.
    Bot,
    /// A message from the bot to the user.
    User,
}

/// A message to be sent between the clients.
pub struct Message {
    /// The action to be taken with the message.
    action: Action,
    /// The recipient of the message.
    recipient: Recipient,
}

impl Message {
    /// Creates a message to be sent from the user to the bot.
    pub fn to_bot() -> Self {
        Self {
            action: Action::default(),
            recipient: Recipient::Bot,
        }
    }

    /// Creates a message to be sent from the bot to the user.
    pub fn to_user() -> Self {
        Self {
            action: Action::default(),
            recipient: Recipient::User,
        }
    }

    /// Gets the action to be taken with the message.
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// Gets the recipient of the message.
    pub fn recipient(&self) -> &Recipient {
        &self.recipient
    }

    /// Unwraps the message into its components.
    pub fn unwrap(self) -> (Action, Recipient) {
        (self.action, self.recipient)
    }

    /// Sends a message to a chat.
    pub fn send_message(mut self, chat: types::Chat, input: types::InputMessage) -> Self {
        self.action = Action::SendMessage(chat, input);
        self
    }

    /// Sends a via bot message to a chat.
    pub fn send_via_bot_message(mut self, chat: types::Chat, input: types::InputMessage) -> Self {
        if self.recipient == Recipient::User {
            panic!("Cannot send a via bot message from the bot to the user");
        }

        self.action = Action::SendViaBotMessage(chat, input);
        self
    }

    /// Edits a message.
    pub fn edit_message(
        mut self,
        chat: types::Chat,
        message_id: i32,
        input: types::InputMessage,
    ) -> Self {
        self.action = Action::EditMessage(chat, message_id, input);
        self
    }
}

async fn handle_message(
    bot: grammers_client::Client,
    user: grammers_client::Client,
    mut rx: Receiver,
    bot_ctx: Context,
) -> Result<()> {
    let bot_me = bot.get_me().await?;
    let bot_username = bot_me.username().unwrap().to_owned();

    let bot_chat = user.resolve_username(&bot_username).await?.unwrap();

    while let Some(message) = rx.recv().await {
        let (action, recipient) = message.unwrap();

        match action {
            Action::SendMessage(chat, input) => {
                match recipient {
                    Recipient::Bot => {
                        // Sends the message to the bot.
                        bot.send_message(chat, input).await?;
                    }
                    Recipient::User => {
                        // Sends the message to the user.
                        user.send_message(chat, input).await?;
                    }
                }
            }
            Action::SendViaBotMessage(chat, input) => {
                let number = rand::random::<i64>();

                let bot_chat = bot_chat.clone();
                let client = user.clone();
                tokio::task::spawn(async move {
                    let mut results = client
                        .inline_query(&bot_chat, &number.to_string())
                        .chat(&chat);

                    loop {
                        match results.next().await {
                            Ok(Some(result)) => {
                                let title = result.title().unwrap();

                                if *title == number.to_string() {
                                    result.send(&chat).await.unwrap();
                                }

                                break;
                            }
                            Ok(None) => tokio::time::sleep(Duration::from_secs(1)).await,

                            Err(e) if e.is("BOT_RESPONSE_TIMEOUT") => {
                                tokio::time::sleep(Duration::from_secs(1)).await
                            }
                            Err(e) => {
                                log::error!("Error: {}", e);
                                break;
                            }
                        }
                    }
                });

                loop {
                    if let Ok(query) = bot_ctx.wait_for_inline_query(Some(10)).await {
                        if query.text() == number.to_string() {
                            query
                                .answer(vec![inline::query::Article::new(
                                    number.to_string(),
                                    input,
                                )
                                .into()])
                                .send()
                                .await
                                .unwrap();

                            break;
                        }
                    }
                }
            }
            Action::EditMessage(chat, message_id, input) => {
                match recipient {
                    Recipient::Bot => {
                        // Edits the message from the bot.
                        bot.edit_message(chat, message_id, input).await?;
                    }
                    Recipient::User => {
                        // Edits the message from the user.
                        user.edit_message(chat, message_id, input).await?;
                    }
                }
            }
            Action::Undefined => {
                log::error!("Undefined action");
            }
        }
    }

    Ok(())
}
