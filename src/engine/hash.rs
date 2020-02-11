//! The Transposition Table
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::num::NonZeroU16;
use std::convert::TryFrom;
use std::mem::size_of;
use crate::chess::{Square, Promotion, Move, ValidMove, MoveBuilder, Position, Zobrist};
use crate::chess::Result;
use crate::engine::Score;


////////////////////////////////////////////////////////////////////////////////////////////////////
/// A representation of a move that fits in 16 bits.
///
/// `Option<HashMove>` is also guaranteed to be only 16 bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HashMove(NonZeroU16);

impl HashMove {
    pub fn origin(self) -> Square {
        Square::try_from(((self.0.get() >> 9) & 0o77) as usize).expect("INFALLIBLE")
    }
    pub fn destination(self) -> Square {
        Square::try_from(((self.0.get() >> 3) & 0o77) as usize).expect("INFALLIBLE")
    }
    pub fn promotion(self) -> Option<Promotion> {
        match self.0.get() & 0o7 {
            0 => None,
            1 => Some(Promotion::ToKnight),
            2 => Some(Promotion::ToBishop),
            3 => Some(Promotion::ToRook),
            4 => Some(Promotion::ToQueen),
            _ => unreachable!(),
        }
    }

    pub fn validate<'a>(&self, pos: &'a Position) -> Result<Move<'a>> {
        MoveBuilder::new()
            .origin(self.origin())
            .destination(self.destination())
            .promotion(self.promotion())
            .validate(pos)
    }
}

impl<T: ValidMove> From<T> for HashMove {
    fn from(mv: T) -> HashMove {
        HashMove(NonZeroU16::new(
            ((mv.origin() as u16) << 9)
            + ((mv.destination() as u16) << 3)
            + match mv.promotion() {
                None => 0,
                Some(prom) => prom as u16,
            }
        ).expect("INFALLIBLE"))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Indicates the kind of bound a transposition table has.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bound {
    Lower,
    Exact,
    Upper,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An entry in the transposition table.
///
/// It is guaranteed to be exactly 16 bytes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HashEntry {
    // 8 bytes
    zobrist: Zobrist,
    // 2 bytes
    when: u16,
    // 1 bytes
    depth: u8,
    // 1 byte
    bound: Bound,
    // 2 bytes
    score: Score,
    // 2 bytes
    best_move: Option<HashMove>,
}

impl HashEntry {
    pub fn new(
        zobrist: Zobrist,
        now: u16, depth: u8,
        bound: Bound, score: Score,
        best_move: HashMove)
    -> HashEntry {
        HashEntry {
            zobrist,
            when: now,
            depth,
            bound,
            score,
            best_move: Some(best_move),
        }
    }

    pub fn new_without_move(
        zobrist: Zobrist,
        now: u16, depth: u8,
        bound: Bound, score: Score)
    -> HashEntry {
        HashEntry {
            zobrist,
            when: now,
            depth,
            bound,
            score,
            best_move: None,
        }
    }

    pub fn zobrist(&self) -> Zobrist {
        self.zobrist
    }

    pub fn when(&self) -> u16 {
        self.when
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn best_move(&self) -> Option<HashMove> {
        self.best_move
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A transposition table
#[derive(Debug)]
pub struct HashTable(Vec<BucketList>);
type BucketList = [Option<HashEntry>; HashTable::BUCKETS];

impl HashTable {
    const BUCKETS: usize = 4;

    pub fn new(size: usize) -> HashTable {
        let size = (size/2).next_power_of_two();
        let elems = size/size_of::<BucketList>();

        HashTable(vec![[None; Self::BUCKETS]; elems])
    }

    pub fn get(&self, zobrist: Zobrist, cur_ply: usize) -> Option<HashEntry> {
        let index = u64::from(zobrist) as usize & (self.0.len() - 1);

        for bucket in 0..Self::BUCKETS {
            match self.0[index][bucket] {
                Some(mut entry) if zobrist == entry.zobrist => {
                    if entry.score >= Score::mates_in(1_000) {
                        entry.score = entry.score + cur_ply as i16;
                    } else if entry.score <= Score::mated_in(1_000) {
                        entry.score = entry.score - cur_ply as i16;
                    }

                    return Some(entry);
                },
                _ => { },
            }
        }

        None
    }

    pub fn insert(&mut self, mut new_entry: HashEntry, cur_ply: usize) {
        let index = u64::from(new_entry.zobrist) as usize & (self.0.len() - 1);

        if new_entry.score >= Score::mates_in(1_000) {
            new_entry.score = new_entry.score - cur_ply as i16;
        } else if new_entry.score <= Score::mated_in(1_000) {
            new_entry.score = new_entry.score + cur_ply as i16;
        }

        let mut draft = 0;
        let mut bucket = 0;
        for b in 0..Self::BUCKETS {
            match self.0[index][b] {
                Some(entry) if new_entry.zobrist == entry.zobrist => {
                    bucket = b;
                    break;
                },
                Some(entry) => {
                    let cur_draft = entry.depth as u16 + entry.when;
                    if cur_draft > draft {
                        draft = cur_draft;
                        bucket = b;
                    }
                },
                None => {
                    draft = u16::max_value();
                    bucket = b;
                },
            }
        }

        self.0[index][bucket] = Some(new_entry);
    }

    pub fn clear(&mut self) {
        let len = self.0.len();
        self.0.clear();
        self.0.resize(len, [None; Self::BUCKETS]);
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_entry_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<Option<HashEntry>>(), 16);
    }
}
