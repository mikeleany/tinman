//! The engine
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////

use std::cmp::max;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::convert::TryInto;
use log::debug;
use crate::chess;
use chess::{Position, ValidMove, Piece};
use chess::game::{MoveSequence, TimeControl};
use crate::protocol::{Protocol, SearchAction};

mod eval;
use eval::{evaluate, piece_val};
pub use eval::Score;

mod hash;
use hash::{HashTable, HashEntry, Bound};

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Thinking output
#[derive(Debug, Clone)]
pub struct Thinking {
    score: Score,
    depth: u8,
    time: Duration,
    nodes: u64,
    pv: MoveSequence,
}

impl Thinking {
    fn new(pos: Arc<Position>) -> Self {
        Thinking {
            score: -Score::infinity(),
            depth: 0,
            time: Duration::from_secs(0),
            nodes: 1,
            pv: MoveSequence::starting_at(pos),
        }
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
    pub fn pv(&self) -> &MoveSequence {
        &self.pv
    }

    /// Returns the best move found in the search.
    pub fn best_move(&self) -> Option<&chess::ArcMove> {
        self.pv.first()
    }

    /// Returns the best move to ponder on.
    pub fn ponder_move(&self) -> Option<&chess::ArcMove> {
        self.pv.get(1)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// The engine
#[derive(Debug)]
pub struct Engine<T> where T: Protocol {
    protocol: T,
    hash: HashTable,

    max_depth: Option<u8>,
    start_time: Instant,
    stop_times: Option<(Instant, Instant)>,
    pondering: bool,
    abort: bool,
    nodes: u64,
    search_count: u16,

    history: MoveSequence,
    color: chess::Color,
}

impl<T> Engine<T> where T: Protocol {
    const DEFAULT_HASH_SIZE: usize = 0x0100_0000; // default to 16 MB hash

    /// Creates a new Engine.
    pub fn new(protocol: T) -> Self {
        Engine {
            protocol,
            hash: HashTable::new(Self::DEFAULT_HASH_SIZE),
            max_depth: None,
            start_time: Instant::now(),
            stop_times: None,
            pondering: false,
            abort: false,
            nodes: 1,
            search_count: 0,
            history: MoveSequence::new(),
            color: chess::Color::White,
        }
    }

    /// Runs the engine.
    ///
    /// # Panics
    ///
    /// Panics if a ponder move returned by the protocol, is not legal.
    pub fn run(&mut self) {
        while self.protocol.wait_for_search() {
            self.start_time = Instant::now();
            self.abort = false;

            self.history = self.protocol.game().history().clone();

            if let Some(mv) = self.protocol.ponder_move() {
                self.pondering = true;
                self.history.push(mv.clone()).expect("Ponder move must be legal");
                debug!("pondering");
            } else {
                self.pondering = false;
                self.calc_search_time();
            }
            self.color = self.history.final_position().turn();

            if let Some(thinking) = self.search_root() {
                if self.pondering {
                    loop {
                        match self.protocol.check_input() {
                            Some(SearchAction::Abort) => { },
                            Some(_) => self.protocol.send_move(&thinking),
                            None => { continue },
                        }
                        break;
                    }
                } else {
                    self.protocol.send_move(&thinking);
                }
            }
        }
    }

    /// Calculate the amount of time that the engine should search.
    fn calc_search_time(&mut self) {
        use TimeControl::*;
        let clock = self.protocol.game().clock();

        match clock.time_control() {
            Infinite => {
                debug!("time remaining: infinite");

                self.stop_times = None;
            },
            Exact(time) => {
                debug!("time remaining: {:?}", time);

                let stop_time = self.start_time + time;
                self.stop_times = Some((stop_time, stop_time));
            },
            Incremental{ inc, .. } => {
                let time = clock.remaining(self.color);
                debug!("time remaining: {:?}", time);
                debug!("time increment: {:?}", inc);

                let search_time = if time > inc * 6 {
                    time/30 + inc
                } else {
                    time/5
                };
                self.stop_times = Some((
                    self.start_time + search_time,
                    self.start_time + search_time * 2
                ));
            },
            _ => {
                let time = clock.remaining(self.color);
                let search_time = time / 30;
                debug!("time remaining: {:?}", time);

                self.stop_times = Some((
                    self.start_time + search_time,
                    self.start_time + search_time * 2
                ));
            },

        }
    }

    /// Search the current or ponder position for the best move, returning the thinking ouptput.
    fn search_root(&mut self) -> Option<Thinking> {
        let mut thinking = Thinking::new(Arc::clone(self.history.final_position()));
        let mut move_list: VecDeque<MoveSequence> = VecDeque::new();
        self.search_count += 1;
        self.nodes = 1;

        // make and store all legal moves
        debug!("searching: {}", self.history.final_position());
        for mv in self.history.final_position().moves() {
            if let Ok(seq) = std::iter::once(mv.into()).collect() {
                move_list.push_back(seq);
            }
        }

        // if no legal moves
        if move_list.is_empty() {
            // TODO: how should we really handle this?
            return Some(thinking);
        }

        // iterative deepening
        let mut best_move = 0;
        let max_depth = if move_list.len() > 1 {
            self.max_depth.unwrap_or(u8::max_value())
        } else {
            2
        };
        for depth in 1 ..= max_depth {
            let mut best_val = -Score::infinity();

            if best_move > 0 {
                // put previous best move at the front
                let mv = move_list.remove(best_move).expect("INFALLIBLE");
                move_list.push_front(mv);
            }

            // search each move
            for (n, seq) in move_list.iter().enumerate() {
                self.history.append(&mut seq.clone()).expect("INFALLIBLE");
                if let Some((val, mut new_pv))
                    = self.search(1, depth-1, -Score::infinity(), -best_val) {

                    self.history.pop();
                    let val = -val;

                    if val > best_val {
                        best_val = val;
                        best_move = n;
                        thinking.score = best_val;
                        thinking.depth = depth;
                        thinking.pv = seq.clone();
                        thinking.pv.append(&mut new_pv).expect("INFALLIBLE");
                    }
                } else if self.abort {
                    return None;
                } else {
                    thinking.depth = depth;
                    thinking.time = self.start_time.elapsed();
                    thinking.nodes = self.nodes;
                    return Some(thinking);
                }
            }

            thinking.depth = depth;
            thinking.time = self.start_time.elapsed();
            thinking.nodes = self.nodes;
            self.protocol.send_thinking(&thinking);
        }

        thinking.time = self.start_time.elapsed();
        thinking.nodes = self.nodes;

        Some(thinking)
    }

    /// Search the current search position to the given depth (assuming no extensions or pruning),
    /// looking for a maximum score of `beta` and a minumum score of `alpha`. Returns the score and
    /// principle variation of the best move.
    fn search(&mut self,
        ply: usize, mut depth: u8,
        mut alpha: Score, beta: Score)
    -> Option<(Score, MoveSequence)> {
        let pos = Arc::clone(self.history.final_position());
        let mut pv = MoveSequence::starting_at(Arc::clone(&pos));

        if self.time_to_stop() {
            return None;
        }

        if pos.fifty_moves() || self.history.repetition() {
            return Some((Score::draw(), pv));
        }

        // check extension
        if pos.in_check() {
            depth += 1;
        }

        // transposition table lookup
        if let Some(hash) = self.hash.get(pos.zobrist_key(), ply) {
            if hash.depth() >= depth {
                if hash.score() >= beta && hash.bound() != Bound::Upper {
                    return Some((hash.score(), pv));
                } else if hash.score() <= alpha && hash.bound() != Bound::Lower {
                    return Some((hash.score(), pv));
                } else if hash.bound() == Bound::Exact {
                    // alpha < score < beta due to previous conditions
                    if let Some(mv) = hash.best_move() {
                        if let Ok(mv) = mv.validate(&pos) {
                            pv.push(mv.into());
                        }
                    }

                    return Some((hash.score(), pv));
                }
            } else {
                // TODO: search hash move first
            }
        }

        // leaf node
        if depth == 0 {
            if let Some(score) = self.qsearch(&pos, alpha, beta) {
                return Some((score, pv));
            } else {
                return None;
            }
        }

        // search each move
        let mut best_val = -Score::infinity();
        for mv in pos.moves() {
            if self.history.push(mv.into()).is_ok() {
                if let Some((val, mut new_pv)) = self.search(ply+1, depth-1, -beta, -alpha) {
                    let mv = self.history.pop().expect("INFALLIBLE");
                    let val = -val;

                    if val >= beta {
                        let hash_entry = HashEntry::new(
                            pos.zobrist_key(),
                            self.search_count, depth,
                            Bound::Lower, val,
                            mv.into());
                        self.hash.insert(hash_entry, ply);
                        return Some((val, pv));
                    }

                    best_val = max(best_val, val);
                    if best_val > alpha {
                        alpha = best_val;
                        pv = mv.try_into().expect("INFALLIBLE");
                        pv.append(&mut new_pv).expect("INFALLIBLE");
                    }
                } else {
                    return None;
                }
            }
        }

        let hash_entry;
        if best_val == -Score::infinity() {
            // no moves found
            if pos.in_check() {
                best_val = Score::mated_in(ply);
            } else {
                best_val = Score::draw();
            }
            hash_entry = HashEntry::new_without_move(
                pos.zobrist_key(),
                self.search_count, depth,
                Bound::Exact, best_val);
        } else if let Some(mv) = pv.first() {
            // pv node
            hash_entry = HashEntry::new(
                pos.zobrist_key(),
                self.search_count, depth,
                Bound::Exact, best_val,
                mv.clone().into());
        } else {
            // all node
            hash_entry = HashEntry::new_without_move(
                pos.zobrist_key(),
                self.search_count, depth,
                Bound::Upper, best_val);
        }
        self.hash.insert(hash_entry, ply);

        Some((best_val, pv))
    }

    /// Search all material-gaining moves from the current search position looking for a maximum
    /// score of `beta` and a minumum score of `alpha`. Returns the estimated score for the
    /// either the best move searched or the current search position, whichever is better.
    fn qsearch(&mut self, pos: &Position, mut alpha: Score, beta: Score) -> Option<Score> {
        let eval = evaluate(pos);
        let mut best_val = eval;

        if self.time_to_stop() {
            return None;
        }

        if eval >= beta {
            return Some(eval);
        }

        alpha = max(alpha, best_val);

        // check if its even possible to improve on alpha
        if eval + 2*piece_val(Piece::Queen) <= alpha {
            return Some(eval + 2*piece_val(Piece::Queen));
        }

        for mv in pos.promotions_and_captures() {
            // check if it's no longer possible to raise alpha
            if !mv.is_promotion() {
                let max_val = eval + piece_val(mv.captured_piece().expect("INFALLIBLE"));
                if max_val < alpha {
                    return Some(max(best_val, max_val));
                }
            }

            if let Ok(new_pos) = mv.make() {
                if let Some(val) = self.qsearch(&new_pos, -beta, -alpha) {
                    let val = -val;

                    if val >= beta {
                        return Some(val);
                    }

                    best_val = max(best_val, val);
                    alpha = max(alpha, best_val);
                } else {
                    return None;
                }
            }
        }

        Some(best_val)
    }

    /// Check if it's time to stop. Should be called exactly once per node.
    fn time_to_stop(&mut self) -> bool {
        self.nodes += 1;

        if self.nodes%1000 == 0 {
            match self.protocol.check_input() {
                Some(SearchAction::PonderHit) => {
                    debug!("ponder hit");
                    self.pondering = false;
                    self.calc_search_time();
                }
                Some(SearchAction::Stop) => {
                    debug!("search stopped");
                    self.pondering = false;
                    return true;
                }
                Some(SearchAction::Abort) => {
                    debug!("search aborted");
                    self.abort = true;
                    return true;
                },
                None => { },
            }

            if !self.pondering {
                match self.stop_times {
                    Some((stop_time, _)) if Instant::now() >= stop_time => return true,
                    _ => return false,
                }
            }
        }

        false
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An engine error
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Error;
