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
use std::sync::mpsc;
use chess::Position;
use chess::game::{Game, TimeControl, MoveSequence, GameResult, WinReason};
use log::warn;


////////////////////////////////////////////////////////////////////////////////////////////////////
/// Possible responses to `EngineInterface::go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineResponse {
    /// The engine plays the given move.
    Move(chess::Move<Arc<chess::Position>>),
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

impl From<std::io::Error> for EngineError {
    fn from(_: std::io::Error) -> EngineError {
        EngineError::IOError
    }
}

impl From<mpsc::RecvError> for EngineError {
    fn from(_: mpsc::RecvError) -> EngineError {
        EngineError::IOError
    }
}

impl From<mpsc::RecvTimeoutError> for EngineError {
    fn from(error: mpsc::RecvTimeoutError) -> EngineError {
        use mpsc::RecvTimeoutError::*;
        match error {
            Disconnected => EngineError::IOError,
            Timeout => EngineError::OutOfTime,
        }
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
    fn new_game(&mut self, game: &Game<Arc<Position>>) -> Result<(), EngineError>;

    /// The engine should update it's board to include any moves in `game` that it has not already
    /// seen. The caller must guarantee that no previous moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn send_moves(&mut self, game: &Game<Arc<Position>>) -> Result<(), EngineError>;

    /// The engine should give a response within the available time for the player on move. The
    /// caller must guarantee that all moves in `game` have already been seen by the engine and that
    /// no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn go(&mut self, game: &Game<Arc<Position>>) -> Result<EngineResponse, EngineError>;

    /// The engine should make the last move in `game` and give a response within the available time
    /// for the player next player. The caller must guarantee the last, and only the last move has
    /// not yet been seen by the engine and that no moves have been undone.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn send_move_and_go(&mut self, game: &Game<Arc<Position>>)
    -> Result<EngineResponse, EngineError>;

    /// Sends the result of the game to the engine. The caller must guarantee that all moves have
    /// already been seen by the engine, that no moves have been undone, and that the game has a
    /// final result.
    ///
    /// # Panics
    /// The implementation may panic if the guarantees are not met.
    fn result(&mut self, game: &Game<Arc<Position>>) -> Result<(), EngineError>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A structure which allows one or more games between two engines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameSetup<> {
    tc: TimeControl,
    opening: MoveSequence<Arc<Position>>,
}

impl GameSetup {
    /// Creates a new `GameSetup` object with default starting position and time control.
    pub fn new() -> Self {
        GameSetup{
            tc: TimeControl::default(),
            opening: MoveSequence::default(),
        }
    }

    /// Sets the time control for the game.
    pub fn time_control(&mut self, tc: TimeControl) -> &Self {
        self.tc = tc;

        self
    }

    /// Sets the initial position for the game.
    pub fn initial_pos(&mut self, pos: chess::Position) -> &Self {
        self.opening = MoveSequence::starting_at(Arc::new(pos));

        self
    }

    /// Sets the intial position and opening moves for the game.
    pub fn opening(&mut self, moves: MoveSequence<Arc<Position>>) -> &Self {
        self.opening = moves;

        self
    }

    /// Uses the given engines to play the game that has been set up.
    pub fn play_game(&self,
        mut white: Box<dyn EngineInterface>,
        mut black: Box<dyn EngineInterface>)
    -> (Game<Arc<Position>>, Result<(), EngineError>) {
        let mut game = Game::starting_at(self.opening.initial_position().to_owned());
        game.set_time_control(self.tc);
        for mv in self.opening.iter() {
            game.make_move(mv.to_owned()).expect("INFALLIBLE");
        }

        if let Err(error) = white.new_game(&game) {
            return (game, Err(error));
        }
        if let Err(error) = black.new_game(&game) {
            return (game, Err(error));
        }

        let start = Instant::now();
        let response = if game.position().turn() == chess::Color::White {
            white.go(&game)
        } else {
            black.go(&game)
        };
        match response {
            Ok(EngineResponse::Move(mv)) => {
                if let Err(error) = game.make_move_timed(mv, Instant::now() - start) {
                    game.set_result(GameResult::Win(
                        !game.position().turn(),
                        Some(WinReason::Forfeiture)));
                    warn!("{}", error);
                    return (game, Err(error.into()));
                }
            },
            Ok(EngineResponse::Resignation) => {
                game.set_result(GameResult::Win(
                    !game.position().turn(),
                    Some(WinReason::Resignation)));
            },
            Err(error) => {
                game.set_result(GameResult::Win(
                    !game.position().turn(),
                    Some(WinReason::Forfeiture)));
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
                    if let Err(error) = game.make_move_timed(mv, Instant::now() - start) {
                        game.set_result(GameResult::Win(
                            !game.position().turn(),
                            Some(WinReason::Forfeiture)));
                        warn!("{}", error);
                        return (game, Err(error.into()));
                    }
                },
                Ok(EngineResponse::Resignation) => {
                    game.set_result(GameResult::Win(
                        !game.position().turn(),
                        Some(WinReason::Resignation)));
                },
                Err(EngineError::OutOfTime) => {
                    game.set_result(GameResult::Win(
                        !game.position().turn(),
                        Some(WinReason::Time)));
                }
                Err(error) => {
                    game.set_result(GameResult::Win(
                        !game.position().turn(),
                        Some(WinReason::Forfeiture)));
                    warn!("{}", error);
                    return (game, Err(error));
                },
            }
        }

        let _ = white.result(&game);
        let _ = black.result(&game);

        (game, Ok(()))
    }
}
