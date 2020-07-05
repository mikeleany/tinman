//! Supported chess protocols
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

use std::sync::Arc;
use std::time::Duration;
use chess::game::Game;
use chess::{Move, Position};
use chess::game::MoveSequence;

pub mod client;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Score
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Score {
    /// The score in centipawns.
    Val(i16),
    /// If positive, the engine mates in the given number of plies. If zero or negative, the engine
    /// is mated in the given number of plies (taking the absolute value).
    MateIn(i16),
}

impl From<Score> for i16 {
    fn from(score: Score) -> Self {
        match score {
            Score::MateIn(plies) if plies > 0 => 10_000 - plies as i16,
            Score::Val(val) => val as i16,
            Score::MateIn(plies) /* plies <= 0 */ => -10_000 - plies as i16,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Thinking output
#[derive(Debug, Clone)]
pub struct Thinking {
    score: Score,
    depth: u8,
    time: Duration,
    nodes: u64,
    pv: Option<MoveSequence<Arc<Position>>>,
}

impl Thinking {
    /// Returns a `Thinking` struct that represents no thinking done, and with a score of
    /// `MateIn(0)`.
    pub fn new() -> Thinking {
        Thinking {
            score: Score::MateIn(0),
            depth: 0,
            time: Duration::from_secs(0),
            nodes: 0,
            pv: None,
        }
    }

    /// Set the principle variation, score and depth.
    pub fn set_pv(&mut self, pv: MoveSequence<Arc<Position>>, score: Score) {
        self.score = score;
        self.pv = Some(pv);
    }

    /// Set the depth searched.
    pub fn set_depth(&mut self, depth: u8) {
        self.depth = depth;
    }

    /// Set the amount of time spent searching.
    pub fn set_time(&mut self, time: Duration) {
        self.time = time;
    }

    /// Set the number of nodes searched.
    pub fn set_nodes(&mut self, nodes: u64) {
        self.nodes = nodes;
    }

    /// Returns the estimated score for the principle variation.
    pub fn score(&self) -> Score {
        self.score
    }

    /// Returns the search depth that was reached.
    pub fn depth(&self) -> usize {
        self.depth as usize
    }

    /// Returns the amount of time used for the search.
    pub fn time(&self) -> Duration {
        self.time
    }

    /// Returns the number of nodes searched.
    pub fn nodes(&self) -> u64 {
        self.nodes
    }

    /// Returns the average number of nodes searched per second.
    pub fn nps(&self) -> u64 {
        self.nodes/self.time.as_secs()
    }

    /// Returns the principle variation.
    pub fn pv(&self) -> Option<&MoveSequence<Arc<Position>>> {
        self.pv.as_ref()
    }

    /// Returns the best move found in the search.
    pub fn best_move(&self) -> Option<&Move<Arc<Position>>> {
        self.pv.as_ref()?.first()
    }

    /// Returns the best move to ponder on.
    pub fn ponder_move(&self) -> Option<&Move<Arc<Position>>> {
        self.pv.as_ref()?.get(1)
    }
}

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
    fn game(&self) -> &Game<Arc<Position>>;

    /// Returns the current ponder move, if any.
    fn ponder_move(&self) -> Option<&Move<Arc<Position>>>;

    /// Returns the maximum search depth (if any)
    fn max_depth(&self) -> Option<usize>;
}

pub mod io;
pub mod xboard;
