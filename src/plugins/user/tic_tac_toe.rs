// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the tic tac toe command handler.

use ferogram::{handler, Context, Result, Router};
use grammers_client::{reply_markup, types::InputMessage};

use crate::{
    filters,
    modules::games::{GameManager, Player, TicTacToe},
    utils::board_to_buttons,
    Sender,
};

/// Setup the tic tac toe command.
pub fn setup() -> Router {
    Router::default()
        .handler(handler::new_message(filters::commands(&["ttt", "tic_tac_toe"])).then(tic_tac_toe))
}

/// Handles the tic tac toe command.
async fn tic_tac_toe(ctx: Context, manager: GameManager, tx: Sender) -> Result<()> {
    let sender = ctx.sender().expect("Sender not found");
    let mut players = vec![Player::new(&sender)];

    if let Some(reply) = ctx.get_reply().await? {
        players.push(Player::new(
            &reply.sender().expect("Reply sender not found"),
        ));
    }

    let mut ttt = TicTacToe::new(manager.new_id(), players);
    ttt.generate_board(3..=3);
    let game = ttt.into_game();

    let buttons = board_to_buttons(game.board(), game.id());
    tx.send(crate::Message::to_bot().send_via_bot_message(
        ctx.chat().expect("Chat not found"),
        InputMessage::html(game.generate_text()).reply_markup(&reply_markup::inline(buttons)),
    ))
    .await?;

    manager.add_game(game);

    Ok(())
}
