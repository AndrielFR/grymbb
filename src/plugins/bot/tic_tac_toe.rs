// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the tic tac toe command handler.

use ferogram::{filter, handler, Result, Router};
use grammers_client::{reply_markup, types::CallbackQuery, InputMessage};
use maplit::hashmap;

use crate::{
    modules::{
        games::{GameManager, Player},
        i18n::I18n,
    },
    utils::board_to_buttons,
};

/// Setup the tic tac toe command.
pub fn setup() -> Router {
    Router::default()
        .handler(handler::callback_query(filter::regex(r"^ttt (\d+)")).then(tic_tac_toe))
}

/// Handles the tic tac toe command.
async fn tic_tac_toe(query: CallbackQuery, i18n: I18n, mut manager: GameManager) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    let data = String::from_utf8(query.data().to_vec())?;
    let split = data.split_whitespace().skip(1).collect::<Vec<_>>();

    let game_id = split[0].parse::<i32>()?;
    if let Some(mut game) = manager.get_game(game_id) {
        let sender = query.sender();

        if !game.has_player(sender.id()) && game.available_seats() == 0 {
            query.answer().alert(t("not_in_game")).send().await?;
            return Ok(());
        } else if !game.has_player(sender.id()) && game.available_seats() > 0 {
            if game.add_player(Player::new(&sender)) {
                query.answer().text(t("joined_game")).send().await?;

                if game.current_player().is_none() {
                    game.set_current_player(sender.id());
                } else {
                    let buttons = board_to_buttons(game.board(), game.id());
                    query
                        .answer()
                        .edit(
                            InputMessage::html(game.generate_text())
                                .reply_markup(&reply_markup::inline(buttons)),
                        )
                        .await?;

                    manager.update_game(game);
                    return Ok(());
                }
            } else {
                query.answer().alert(t("game_full")).send().await?;
                return Ok(());
            }
        } else if let Some(player) = game.current_player() {
            if player.id() != sender.id() {
                query.answer().alert(t("not_your_turn")).send().await?;
                return Ok(());
            }
        } else {
            query.answer().alert(t("not_your_turn")).send().await?;
            return Ok(());
        }

        let column = split[1].parse::<usize>()?;
        let row = split[2].parse::<usize>()?;
        if !game.play(column, row) {
            query.answer().alert(t("ocupied_cell")).send().await?;
            return Ok(());
        }

        let buttons = board_to_buttons(game.board(), game.id());
        let mut answer = query.answer();

        if game.is_over() {
            if let Some(player) = game.winner() {
                if player.id() == sender.id() {
                    answer = answer.alert(t("you_win"));
                } else {
                    answer = answer.alert(t_a(
                        "player_wins",
                        hashmap! { "player" => player.mention() },
                    ));
                }
            } else {
                answer = answer.alert(t("game_draw"));
            }
        }

        answer
            .edit(
                InputMessage::html(game.generate_text())
                    .reply_markup(&reply_markup::inline(buttons)),
            )
            .await?;

        if game.is_over() {
            manager.remove_game(game);
        } else {
            manager.update_game(game);
        }
    } else {
        query.answer().alert(t("game_not_found")).send().await?;
    }

    Ok(())
}
