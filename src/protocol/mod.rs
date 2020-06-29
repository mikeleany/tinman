//! Supported chess protocols
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::engine::Thinking;
use chess::game::Game;
use chess::ArcMove;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An action that should between searches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The engine should exit.
    Quit,
    /// The engine should search the current position.
    Search,
    /// The engine should adjust the size of the transposition table, (given in bytes).
    HashSize(usize),
    /// The engine should clear the transposition table.
    ClearHash,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An action that should be taken regarding the current search.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchAction {
    /// The ponder move was played. The engine should leave ponder mode and continue thinking until
    /// it's ready to make a move.
    PonderHit,

    /// The engine should stop thinking and call
    /// [send_move](trait.Protocol.html#tymethod.send_move).
    Stop,

    /// The engine should stop thinking, but should *not* call
    /// [send_move](trait.Protocol.html#tymethod.send_move).
    Abort,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Trait used for structures that implement the engine side of a chess protocol
pub trait Protocol {
    /// Waits for the next action the engine should take.
    ///
    /// Returns the next action that the engine should take.
    fn wait_for_direction(&mut self) -> Action;

    /// Sends the engine's move to the client. If supported by the protocol, the engine's request to
    /// resign, claim a draw, or offer a draw should be carried out. 
    ///
    /// # TODO
    /// Determine what to do if thinking doesn't contain a move.
    fn send_move(&mut self, thinking: &Thinking);

    /// If supported by the protocol, send the engine's thinking to the client.
    fn send_thinking(&mut self, thinking: &Thinking);

    /// If supported by the protocol, send a debug message to the client.
    fn send_debug_msg(&mut self, msg: &str);

    /// Allows the protocol to check the input during a search.
    ///
    /// If the search should end for any reason, returns how it should end. Returns PonderHit if
    /// the ponder move was made by the client.
    fn check_input(&mut self) -> Option<SearchAction> where Self: Sized;

    /// Returns the current state of the game.
    fn game(&self) -> &Game;

    /// Returns the current ponder move, if any.
    fn ponder_move(&self) -> Option<&ArcMove>;

    /// Returns the maximum search depth (if any)
    fn max_depth(&self) -> Option<usize>;
}

pub mod io;
pub mod xboard;
