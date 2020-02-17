//! Code to run games between two engines.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::protocol::io;
use crate::chess;
use chess::game::{Game, TimeControl, MoveSequence};


////////////////////////////////////////////////////////////////////////////////////////////////////
/// Possible responses to `EngineInterface::go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineResponse {
    /// The engine plays the given move.
    Move(chess::ArcMove),

    /// The engine plays the given move and claims a draw. The engine must be able to continue
    /// playing if the draw claim is rejected.
    ClaimDraw(chess::ArcMove),

    /// The engine resigns. No move is played.
    Resignation,

    /// The engine encountered a fatal error.
    EngineError(String),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Trait for engine interfaces.
pub trait EngineInterface {
    /// The engine should prepare for a new game with the starting conditions given by `game`.
    fn new_game(&mut self, game: &Game);

    /// The engine should update it's internal state to match `game` and give a response within the
    /// available time for the player on move.
    fn go(&mut self, game: &Game) -> EngineResponse;

    /// Provides the engine with final updates on the game state.
    fn result(&mut self, game: &Game);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A structure which allows one or more games between two engines.
pub struct Match<'a> {
    engine: [&'a mut EngineInterface; 2],
    tc: TimeControl,
    opening: MoveSequence,
}

impl<'a> Match<'a> {
    pub fn new(white: &'a mut EngineInterface, black: &'a mut EngineInterface) -> Self {
        unimplemented!()
    }

    pub fn time_control(&mut self, tc: TimeControl) -> &Self {
        unimplemented!()
    }

    pub fn initial_pos(&mut self, pos: chess::Position) -> &Self {
        unimplemented!()
    }

    pub fn opening(&mut self, moves: MoveSequence) -> &Self {
        unimplemented!()
    }

    pub fn play_game(&mut self) -> Game {
        unimplemented!()

        // for each engine
            // begin new game

        // until game over
            // send updates to engine on move
            // until the user moves
                // wait for response (with timeout)
                // parse response
            // make move on board

        // return game
    }
}
