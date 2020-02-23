//! Code to run games between two engines.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::sync::Arc;
use crate::protocol::io;
use crate::chess;
use chess::game::{Game, TimeControl, MoveSequence};


////////////////////////////////////////////////////////////////////////////////////////////////////
/// Possible responses to `EngineInterface::go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineResponse {
    /// The engine plays the given move.
    Move(chess::ArcMove),

    /// The engine resigns. No move is played.
    Resignation,

    /// The engine encountered a fatal error.
    EngineError(String),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Trait for engine interfaces.
pub trait EngineInterface {
    /// The engine should prepare for a new game with the time control, initial position, and moves
    /// given in `game`. The caller must guarantee that the time control is not infinite.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn new_game(&mut self, game: &Game);

    /// The engine should update it's board to include any moves in `game` that it has not already
    /// seen. The caller must guarantee that no previous moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn send_moves(&mut self, game: &Game);

    /// The engine should give a response within the available time for the player on move. The
    /// caller must guarantee that all moves in `game` have already been seen by the engine and that
    /// no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn go(&mut self, game: &Game) -> EngineResponse;

    /// The engine should make the last move in `game` and give a response within the available time
    /// for the player next player. The caller must guarantee the last, and only the last move has
    /// not yet been seen by the engine and that no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn send_move_and_go(&mut self, game: &Game) -> EngineResponse;

    /// Sends the result of the game to the engine. The caller must guarantee that all moves have
    /// already been seen by the engine and that no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn result(&mut self, game: &Game);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A structure which allows one or more games between two engines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameSetup<> {
    tc: TimeControl,
    opening: MoveSequence,
}

impl GameSetup {
    pub fn new() -> Self {
        GameSetup{
            tc: TimeControl::default(),
            opening: MoveSequence::default(),
        }
    }

    pub fn time_control(&mut self, tc: TimeControl) -> &Self {
        self.tc = tc;

        self
    }

    pub fn initial_pos(&mut self, pos: chess::Position) -> &Self {
        self.opening = MoveSequence::starting_at(Arc::new(pos));

        self
    }

    pub fn opening(&mut self, moves: MoveSequence) -> &Self {
        self.opening = moves;

        self
    }

    pub fn play_game(&self,
        white: &mut dyn EngineInterface,
        black: &mut dyn EngineInterface)
    -> Game {
        let mut game = Game::starting_at(self.opening.initial_position().as_ref().to_owned());
        game.set_time_control(self.tc);
        for mv in self.opening.iter() {
            game.make_move(mv.to_owned());
        }

        white.new_game(&game);
        black.new_game(&game);

        let response = if game.position().turn() == chess::Color::White {
            white.go(&game)
        } else {
            black.go(&game)
        };
        match response {
            EngineResponse::Move(mv) => {
                game.make_move(mv);
            },
            _ => { todo!() },
        }

        while game.result().is_none() {
            let response = if game.position().turn() == chess::Color::White {
                white.send_move_and_go(&game)
            } else {
                black.send_move_and_go(&game)
            };
            match response {
                EngineResponse::Move(mv) => {
                    game.make_move(mv);
                },
                _ => { todo!() },
            }
        }

        white.result(&game);
        black.result(&game);

        game
    }
}
