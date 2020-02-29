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
use std::time::Instant;
use std::fmt;
use crate::chess;
use chess::game::{Game, TimeControl, MoveSequence};
use log::warn;


////////////////////////////////////////////////////////////////////////////////////////////////////
/// Possible responses to `EngineInterface::go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineResponse {
    /// The engine plays the given move.
    Move(chess::ArcMove),
    /// The engine resigns. No move is played.
    Resignation,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Error in response to `EngineInterface::go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError{
    /// The engine ran out of time to respond
    OutOfTime,
    /// Lost communication with the engine
    IOError,
    /// The engine tried to make an illegal move
    IllegalMove,
    /// The engine did not conform with the protocol
    ProtocolError,
    /// The engine did not accept a legal move
    RejectedLegalMove,
    /// The engine claims the game is over when it's not
    FalseResultClaim,
    /// Other error
    Other,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EngineError::*;
        match self {
            OutOfTime => { "engine ran out of time" },
            IOError => { "lost communication with engine" },
            IllegalMove => { "engine tried to make an illegal move" },
            ProtocolError => { "engine did not conform with the protocol" },
            RejectedLegalMove => { "engine did not accept a legal move" },
            FalseResultClaim => { "the engine claimed the game is over when it wasn't" },
            Other => { "engine encountered an unknown error" },
        }.fmt(f)
    }
}

impl std::error::Error for EngineError {}

impl From<std::sync::mpsc::RecvError> for EngineError {
    fn from(_: std::sync::mpsc::RecvError) -> EngineError {
        EngineError::IOError
    }
}

impl From<chess::Error> for EngineError {
    fn from(_: chess::Error) -> EngineError {
        EngineError::IllegalMove
    }
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
    fn go(&mut self, game: &Game) -> Result<EngineResponse, EngineError>;

    /// The engine should make the last move in `game` and give a response within the available time
    /// for the player next player. The caller must guarantee the last, and only the last move has
    /// not yet been seen by the engine and that no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn send_move_and_go(&mut self, game: &Game) -> Result<EngineResponse, EngineError>;

    /// Sends the result of the game to the engine. The caller must guarantee that all moves have
    /// already been seen by the engine, that no moves have been undone, and that the game has a
    /// final result.
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
        mut white: Box<dyn EngineInterface>,
        mut black: Box<dyn EngineInterface>)
    -> (Game, Result<(), EngineError>) {
        let mut game = Game::starting_at(self.opening.initial_position().as_ref().to_owned());
        game.set_time_control(self.tc);
        for mv in self.opening.iter() {
            game.make_move(mv.to_owned());
        }

        white.new_game(&game);
        black.new_game(&game);

        let start = Instant::now();
        let response = if game.position().turn() == chess::Color::White {
            white.go(&game)
        } else {
            black.go(&game)
        };
        match response {
            Ok(EngineResponse::Move(mv)) => {
                game.make_move_timed(mv, Instant::now() - start);
            },
            Ok(_) => { todo!() },
            Err(error) => {
                // TODO: set the game result
                warn!("{}", error);
                return (game, Err(error));
            },
        }

        while game.result().is_none() {
            let start = Instant::now();
            let response = if game.position().turn() == chess::Color::White {
                white.send_move_and_go(&game)
            } else {
                black.send_move_and_go(&game)
            };
            match response {
                Ok(EngineResponse::Move(mv)) => {
                    game.make_move_timed(mv, Instant::now() - start);
                },
                Ok(_) => { todo!() },
                Err(error) => {
                    // TODO: set the game result
                    warn!("{}", error);
                    return (game, Err(error));
                },
            }
        }

        white.result(&game);
        black.result(&game);

        (game, Ok(()))
    }
}
