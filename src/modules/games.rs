// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the games module.

use std::{collections::HashMap, ops::RangeInclusive, sync::Arc};

use grammers_client::types::Chat;
use tokio::sync::Mutex;

/// The symbols.
const SYMBOLS: [char; 3] = ['⭕', '❌', '🟥'];

/// The game manager.
#[derive(Clone)]
pub struct GameManager {
    /// The active games.
    active_games: Arc<Mutex<Vec<Game>>>,
}

impl GameManager {
    /// Creates a new `GameManager` instance.
    pub fn new() -> Self {
        Self {
            active_games: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Generates a new game ID.
    pub fn new_id(&self) -> i32 {
        let games = self
            .active_games
            .try_lock()
            .expect("failed to lock active games");
        let last_id = games.last().map(|g| g.id()).unwrap_or(0);

        last_id + 1
    }

    /// Adds a game to the list of active games.
    pub fn add_game(&self, game: Game) {
        self.active_games
            .try_lock()
            .expect("failed to lock active games")
            .push(game);
    }

    /// Returns the game with the given ID.
    pub fn get_game(&self, game_id: i32) -> Option<Game> {
        self.active_games
            .try_lock()
            .expect("failed to lock active games")
            .iter()
            .find(|g| g.id() == game_id)
            .cloned()
    }

    /// Updates a game.
    pub fn update_game(&mut self, game: Game) {
        let game_id = game.id();
        *self
            .active_games
            .try_lock()
            .expect("failed to lock active games")
            .iter_mut()
            .find(|g| g.id() == game_id)
            .expect("failed to find game") = game;
    }

    /// Removes a game from the list of active games.
    pub fn remove_game(&self, game: Game) {
        self.active_games
            .try_lock()
            .expect("failed to lock active games")
            .retain(|g| g.id() != game.id());
    }
}

/// The game.
#[derive(Clone)]
pub enum Game {
    /// The tic tac toe game.
    TicTacToe(TicTacToe),
    /* /// The sudoku game.
    Sudoku(Sudoku), */
}

impl Game {
    /// Returns the game ID.
    pub fn id(&self) -> i32 {
        match self {
            Self::TicTacToe(g) => g.id,
        }
    }

    /// Plays the game.
    pub fn play(&mut self, column: usize, row: usize) -> bool {
        match self {
            Self::TicTacToe(g) => {
                if let Some(player) = g.players.get(&g.current_player) {
                    let symbol = player.symbol();

                    if g.board[column][row] == SYMBOLS[2] {
                        g.board[column][row] = symbol;

                        let mut winner = None;

                        // X
                        // X
                        // X
                        for row in &g.board {
                            if row.iter().all(|s| *s == symbol) {
                                winner = Some(player.id());
                            }
                        }

                        let board_size = g.board.len();

                        // X X X
                        for i in 0..board_size {
                            if g.board.iter().all(|row| row[i] == symbol) {
                                winner = Some(player.id());
                            }
                        }

                        // X - -
                        // - X -
                        // - - X
                        if (0..board_size).all(|i| g.board[i][i] == symbol) {
                            winner = Some(player.id());
                        }

                        // - - X
                        // - X -
                        // X - -
                        if (0..board_size).all(|i| g.board[board_size - i - 1][i] == symbol) {
                            winner = Some(player.id());
                        }

                        if let Some(id) = winner {
                            g.winner = Some(id);
                            g.state = State::End;
                        } else {
                            if g.board
                                .iter()
                                .all(|row| row.iter().all(|s| *s != SYMBOLS[2]))
                            {
                                g.state = State::End;
                            }
                        }

                        self.switch_player();

                        return true;
                    }
                }

                false
            }
        }
    }

    /// Returns the game board.
    pub fn board(&self) -> Vec<Vec<char>> {
        match self {
            Self::TicTacToe(g) => g.board.clone(),
        }
    }

    /// Returns the game players.
    pub fn players(&self) -> Vec<Player> {
        match self {
            Self::TicTacToe(g) => g.players.clone().into_values().into_iter().collect(),
        }
    }

    /// Checks if the game is over.
    pub fn is_over(&self) -> bool {
        match self {
            Self::TicTacToe(g) => g.state == State::End,
        }
    }

    /// Returns the winner of the game.
    pub fn winner(&self) -> Option<&Player> {
        match self {
            Self::TicTacToe(g) => self.get_player(g.winner?),
        }
    }

    /// Adds a player to the game.
    ///
    /// Returns `true` if the player was added, `false` otherwise.
    pub fn add_player(&mut self, mut player: Player) -> bool {
        let limit = self.players_limit();

        match self {
            Self::TicTacToe(g) => {
                if g.players.contains_key(&player.id()) {
                    return false;
                } else if g.players.len() >= limit {
                    return false;
                }

                player.symbol = SYMBOLS[1];
                g.players.insert(player.id(), player);
                g.state = State::Playing;

                true
            }
        }
    }

    /// Returns the player with the given ID.
    pub fn get_player(&self, id: i64) -> Option<&Player> {
        match self {
            Self::TicTacToe(g) => g.players.get(&id),
        }
    }

    /// Checks if the player is in the game.
    pub fn has_player(&self, id: i64) -> bool {
        match self {
            Self::TicTacToe(g) => g.players.contains_key(&id),
        }
    }

    #[allow(dead_code)]
    /// Removes a player from the game.
    pub fn remove_player(&mut self, id: i64) {
        match self {
            Self::TicTacToe(g) => {
                g.players.remove(&id);
            }
        }
    }

    /// Returns the current player.
    pub fn current_player(&self) -> Option<&Player> {
        match self {
            Self::TicTacToe(g) => g.players.get(&g.current_player),
        }
    }

    #[allow(dead_code)]
    /// Returns the next player.
    pub fn next_player(&self) -> Option<&Player> {
        match self {
            Self::TicTacToe(g) => {
                let next_player = g.players.keys().find(|id| **id != g.current_player)?;

                g.players.get(next_player)
            }
        }
    }

    /// Returns the player list.
    pub fn player_list(&self) -> String {
        let mut text = String::new();

        let winner_id = if let Some(player) = self.winner() {
            player.id()
        } else {
            0
        };

        match self {
            Self::TicTacToe(g) => {
                for (i, (player_id, player)) in g.players.iter().enumerate() {
                    if *player_id == winner_id {
                        text += &format!("👑 <b>{0}</b> ({1})", player.mention(), player.symbol());
                    } else if g.state == State::End {
                        text += &format!("🤡 <s>{0}</s> ({1})", player.mention(), player.symbol());
                    } else if *player_id == g.current_player {
                        text += &format!("<u>{0}</u> ({1})", player.mention(), player.symbol());
                    } else {
                        text += &format!("{0} ({1})", player.mention(), player.symbol());
                    }

                    if i < g.players.len() - 1 {
                        text.push_str(" vs ");
                    }
                }
            }
        }

        text
    }

    /// Returns the players limit.
    pub fn players_limit(&self) -> usize {
        match self {
            Self::TicTacToe(_) => 2,
        }
    }

    /// Generates the game text.
    pub fn generate_text(&self) -> String {
        let mut text = match self {
            Self::TicTacToe(_) => "<b>Tic Tac Toe</b>\n",
        }
        .to_string();
        text += &format!("\n{}", self.player_list());

        text
    }

    #[allow(dead_code)]
    /// Generates a new board.
    pub fn generate_board(&mut self, size: RangeInclusive<usize>) {
        match self {
            Self::TicTacToe(g) => g.generate_board(size),
        }
    }

    /// Switches the current player.
    pub fn switch_player(&mut self) {
        match self {
            Self::TicTacToe(g) => g.switch_player(),
        }
    }

    /// Returns the available seats.
    pub fn available_seats(&self) -> usize {
        self.players_limit() - self.players().len()
    }

    /// Sets the current player.
    pub fn set_current_player(&mut self, id: i64) {
        match self {
            Self::TicTacToe(g) => g.current_player = id,
        }
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TicTacToe(g) => write!(f, "Tic Tac Toe (ID: {})", g.id),
        }
    }
}

/// The game state.
#[derive(Clone, PartialEq)]
pub enum State {
    Start,
    Playing,
    End,
}

/// The tic tac toe game.
#[derive(Clone)]
pub struct TicTacToe {
    /// The game ID.
    id: i32,
    /// The game board.
    board: Vec<Vec<char>>,
    /// The game players.
    players: HashMap<i64, Player>,
    /// The game state.
    state: State,
    /// The game winner.
    winner: Option<i64>,
    /// The last player.
    last_player: i64,
    /// The current player.
    current_player: i64,
}

impl TicTacToe {
    /// Creates a new `TicTacToe` instance.
    pub fn new(id: i32, mut players: Vec<Player>) -> Self {
        let first_player_id = players[0].id();

        for player in &mut players {
            if player.id() == first_player_id {
                player.symbol = SYMBOLS[0];
            } else {
                player.symbol = SYMBOLS[1];
            }
        }

        Self {
            id,
            board: Vec::new(),
            players: players.into_iter().map(|p| (p.id(), p)).collect(),
            state: State::Start,
            winner: None,
            last_player: 0,
            current_player: first_player_id,
        }
    }

    /// Generates a new board.
    pub fn generate_board(&mut self, size: RangeInclusive<usize>) {
        let columns = size.start();
        let rows = size.end();

        let mut board = Vec::with_capacity(*columns);

        for _ in 0..*columns {
            let mut row = Vec::with_capacity(*rows);

            for _ in 0..*rows {
                row.push(SYMBOLS[2]);
            }

            board.push(row);
        }

        self.board = board;
    }

    /// Switches the current player.
    pub fn switch_player(&mut self) {
        if let Some(next_player) = self.players.keys().find(|id| **id != self.current_player) {
            self.last_player = self.current_player;
            self.current_player = *next_player;
        } else {
            self.last_player = self.current_player;
            self.current_player = 0;
        }
    }

    /// Converts tic tac toe into a game.
    pub fn into_game(self) -> Game {
        Game::TicTacToe(self)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Sudoku {
    /// The game ID.
    id: i32,
    /// The game board.
    board: Vec<Vec<char>>,
    /// The game players.
    players: HashMap<i64, Player>,
    /// The game state.
    state: State,
    /// The game winner.
    winner: Option<i64>,
    /// The last player.
    last_player: i64,
    /// The current player.
    current_player: i64,
}

/// The player.
#[derive(Clone)]
pub struct Player {
    /// The player ID.
    id: i64,
    /// The player symbol.
    symbol: char,
    /// The player first name.
    first_name: String,
}

impl Player {
    /// Creates a new `Player` instance.
    pub fn new(user: &Chat) -> Self {
        let id = user.id();
        let symbol = SYMBOLS[id as usize % SYMBOLS.len()];
        let first_name = user.name().to_string();

        Self {
            id,
            symbol,
            first_name,
        }
    }

    /// Returns the player ID.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Returns the player symbol.
    pub fn symbol(&self) -> char {
        self.symbol
    }

    /// Returns the player mention.
    pub fn mention(&self) -> String {
        format!(
            "<a href=\"tg://user?id={0}\">{1}</a>",
            self.id, self.first_name
        )
    }

    #[allow(dead_code)]
    /// Returns the player first name.
    pub fn first_name(&self) -> &str {
        &self.first_name
    }
}
