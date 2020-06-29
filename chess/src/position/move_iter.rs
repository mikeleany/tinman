//! Contains structures to represent and generate moves
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::iter::FusedIterator;
use std::collections::VecDeque;
use super::*;
use Promotion::*;


////////////////////////////////////////////////////////////////////////////////////////////////////
/// An iterator over all valid (pseudo-legal) moves from a position.
///
/// Note that the moves might not be fully legal, specifically, they may leave the mover in check or
/// involve castling through check. Use `Move::make()` to verify full legality.
///
/// Cannot outlive the position it is tied to.
#[derive(Debug, Clone)]
pub struct Moves<'a> {
    pos: &'a Position,
    state: MovesState,
    prom_and_capt: PromotionsAndCaptures<'a>,
    piece: Piece,
    orig: Square,
    board1: Bitboard,
    board2: Bitboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovesState {
    PromAndCapt,
    Castling,
    PawnAdvancement(i8),
    Remaining,
    Finished,
}

impl<'a> Moves<'a> {
    pub (super) fn new(pos: &'a Position) -> Moves {
        Moves {
            pos,
            state: MovesState::PromAndCapt,
            prom_and_capt: PromotionsAndCaptures::new(pos),
            piece: Default::default(),
            orig: Default::default(),
            board1: Default::default(),
            board2: Default::default(),
        }
    }
}

impl<'a> Iterator for Moves<'a> {
    type Item = Move<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use MovesState::*;
        let pos = &self.pos;

        while let PromAndCapt = self.state {
            if let Some(m) = self.prom_and_capt.next() {
                return Some(m);
            } else {
                self.state = MovesState::Castling;
                self.piece = King;
                let rights = if !pos.in_check() {
                    pos.castling_rights[pos.turn() as usize]
                } else {
                    0
                };
                let mask = match rights {
                    CASTLE_KING_SIDE => Bitboard::from(File::H),
                    CASTLE_QUEEN_SIDE => Bitboard::from(File::A),
                    CASTLE_BOTH_SIDES => Bitboard::from(File::A) | File::H.into(),
                    _ => Bitboard::new(),
                };
                self.orig = pos.king_location(pos.turn());
                self.board2 = rank_attacks(self.orig, pos.occupied()) &
                    pos.occupied_by_piece(pos.turn(), Rook) & mask;
            }
        }

        while let Castling = self.state {
            if let Some(rook) = self.board2.pop() {
                let dest_file = match rook.file() {
                    File::H => File::G,
                    File::A => File::C,
                    _ => unreachable!(),
                };
                let dest = Square::from_coord(dest_file, rook.rank());

                return Some(Move {
                    pos,
                    piece: self.piece,
                    orig: self.orig,
                    capt_pc: None,
                    dest,
                    move_type: MoveType::Castling,
                });
            } else {
                let forward = if pos.turn() == White { 1 } else { -1 };
                self.state = PawnAdvancement(forward);
                self.piece = Pawn;
                let pieces = pos.occupied_by_piece(pos.turn(), self.piece);
                let mask = Bitboard::from(Rank::R1) | Rank::R8.into();
                self.board1 = pieces.shift_y(forward) & !pos.occupied() & !mask;
                let adv2_rank = if pos.turn() == White { Rank::R4 } else { Rank::R5 };
                self.board2 = self.board1.shift_y(forward) & !pos.occupied() & adv2_rank.into();
            }
        }

        while let PawnAdvancement(forward) = self.state {
            if let Some(dest) = self.board2.pop() {
                let orig_rank = if pos.turn() == White { Rank::R2 } else { Rank::R7 };
                let orig = Square::from_coord(dest.file(), orig_rank);

                return Some(Move {
                    pos,
                    piece: self.piece,
                    orig,
                    capt_pc: None,
                    dest,
                    move_type: MoveType::Advance2,
                });
            } else if let Some(dest) = self.board1.pop() {
                let orig_rank = Rank::try_from((dest.rank() as i8 - forward) as usize)
                    .expect("INFALLIBLE");
                let orig = Square::from_coord(dest.file(), orig_rank);

                return Some(Move {
                    pos,
                    piece: self.piece,
                    orig,
                    capt_pc: None,
                    dest,
                    move_type: MoveType::Standard,
                });
            } else {
                self.state = Remaining;
                self.piece = Knight;
                self.board1 = pos.occupied_by_piece(pos.turn(), self.piece);
            }
        }

        while let Remaining = self.state {
            if let Some(dest) = self.board2.pop() {
                return Some(Move {
                    pos,
                    piece: self.piece,
                    orig: self.orig,
                    capt_pc: None,
                    dest,
                    move_type: MoveType::Standard,
                });
            } else if let Some(orig) = self.board1.pop() {
                self.orig = orig;
                self.board2 = !pos.occupied() & match self.piece {
                    Pawn => unreachable!(),
                    Knight => knight_attacks(orig),
                    Bishop => bishop_attacks(orig, pos.occupied()),
                    Rook => rook_attacks(orig, pos.occupied()),
                    Queen => queen_attacks(orig, pos.occupied()),
                    King => king_attacks(orig),
                };
            } else if self.piece < King {
                self.piece = match self.piece {
                    Knight => Bishop,
                    Bishop => Rook,
                    Rook => Queen,
                    Queen => King,
                    _ => unreachable!(),
                };
                self.board1 = pos.occupied_by_piece(pos.turn(), self.piece);
            } else {
                self.state = Finished;
            }
        }

        None
    }
}

impl<'a> FusedIterator for Moves<'a> { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An iterator over all valid (pseudo-legal) promotions and captures from a position.
///
/// Note that the moves might not be fully legal, specifically, they may leave the mover in check or
/// involve castling through check. Use `Move::make()` to verify full legality.
///
/// Cannot outlive the position it is tied to.
#[derive(Debug, Clone)]
pub struct PromotionsAndCaptures<'a> {
    pos: &'a Position,
    forward: i8,
    ep_mask: Bitboard,

    under_promotions: VecDeque<Move<'a>>,

    state: PromAndCaptState,
    victim: Piece,
    targets: Bitboard,
    attacker: Piece,
    pieces: Bitboard,
    destinations: Bitboard,

    side: i8,
    orig: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromAndCaptState {
    CapturePromotions,
    Promotions,
    Captures,
    UnderPromotions,
}

impl<'a> PromotionsAndCaptures<'a> {
    pub (super) fn new(pos: &'a Position) -> PromotionsAndCaptures {
        let forward = if pos.turn() == White { 1 } else { -1 };
        let mask = Bitboard::from(Rank::R1) | Rank::R8.into();
        let targets = mask & pos.occupied_by_piece(!pos.turn(), Queen);
        let pieces = pos.occupied_by_piece(pos.turn(), Pawn);
        let destinations = pieces.shift_xy(-1, forward) & targets;

        PromotionsAndCaptures{
            pos,
            forward,
            ep_mask: if let Some(ep_sq) = pos.en_passant_square() {
                ep_sq.into()
            } else {
                Bitboard::new()
            },
            under_promotions: VecDeque::new(),

            state: PromAndCaptState::CapturePromotions,
            victim: Queen,
            targets,
            attacker: Pawn,
            pieces,
            destinations,
            side: -1,

            orig: Default::default(),
        }
    }
}

impl<'a> Iterator for PromotionsAndCaptures<'a> {
    type Item = Move<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use PromAndCaptState::*;
        let pos = &self.pos;

        // --- Capture Promotions ---
        while self.state == CapturePromotions {
            if let Some(dest) = self.destinations.pop() {
                let file = File::try_from((dest.file() as i8 - self.side) as usize)
                    .expect("INFALLIBLE");
                let rank = Rank::try_from((dest.rank() as i8 - self.forward) as usize)
                    .expect("INFALLIBLE");
                let orig = Square::from_coord(file, rank);

                let m = Move{
                    pos: self.pos,
                    piece: self.attacker,
                    orig,
                    capt_pc: Some(self.victim),
                    dest,
                    move_type: MoveType::Promotion(ToQueen),
                };

                for prom_pc in &[ ToKnight, ToRook, ToBishop ] {
                    let mut m = m.clone();
                    m.move_type = MoveType::Promotion(*prom_pc);
                    self.under_promotions.push_back(m);
                }

                return Some(m);
            } else if self.side < 1 {
                self.side = 1;
                self.destinations = self.pieces.shift_xy(self.side, self.forward) & self.targets;
            } else if self.victim > Knight {
                self.victim = match self.victim {
                    Queen => Rook,
                    Rook => Bishop,
                    Bishop => Knight,
                    _ => unreachable!(),
                };
                let mask = Bitboard::from(Rank::R1) | Rank::R8.into();
                self.targets = mask & pos.occupied_by_piece(!pos.turn(), self.victim);
                self.side = -1;
                self.destinations = self.pieces.shift_xy(self.side, self.forward) & self.targets;
            } else {
                self.state = Promotions;
                let mask = Bitboard::from(Rank::R1) | Rank::R8.into();
                self.targets = mask & !pos.occupied();
                self.destinations = self.pieces.shift_y(self.forward) & self.targets;
            }
        }

        // --- Non-capture Promotions ---
        while self.state == Promotions {
            if let Some(dest) = self.destinations.pop() {
                let rank = Rank::try_from((dest.rank() as i8 - self.forward) as usize)
                    .expect("INFALLIBLE");
                let orig = Square::from_coord(dest.file(), rank);

                let m = Move{
                    pos: self.pos,
                    piece: self.attacker,
                    orig,
                    capt_pc: None,
                    dest,
                    move_type: MoveType::Promotion(ToQueen),
                };

                for prom_pc in &[ ToKnight, ToRook, ToBishop ] {
                    let mut m = m.clone();
                    m.move_type = MoveType::Promotion(*prom_pc);
                    self.under_promotions.push_back(m);
                }

                return Some(m);
            } else {
                self.state = Captures;
                self.victim = Queen;
                self.targets = pos.occupied_by_piece(!pos.turn(), self.victim);
                self.attacker = Pawn;
                self.pieces = pos.occupied_by_piece(pos.turn(), self.attacker);
                self.side = -3;
            }
        }

        // --- Remaining captures ---
        while self.state == Captures {
            if let Some(dest) = self.destinations.pop() {
                if self.attacker == Pawn {
                    // captures by pawn
                    let file = File::try_from((dest.file() as i8 - self.side) as usize)
                        .expect("INFALLIBLE");
                    let rank = Rank::try_from((dest.rank() as i8 - self.forward) as usize)
                        .expect("INFALLIBLE");
                    let orig = Square::from_coord(file, rank);
                    let move_type = match pos.en_passant_square() {
                        Some(ep_sq) if dest == ep_sq => MoveType::EnPassant,
                        _ => MoveType::Standard,
                    };

                    return Some(Move {
                        pos: self.pos,
                        piece: self.attacker,
                        orig,
                        capt_pc: Some(self.victim),
                        dest,
                        move_type,
                    })
                } else {
                    return Some(Move {
                        pos: self.pos,
                        piece: self.attacker,
                        orig: self.orig,
                        capt_pc: Some(self.victim),
                        dest,
                        move_type: MoveType::Standard,
                    })
                }
            } else if self.attacker == Pawn && self.side < 1 {
                // switch direction of pawn captures
                self.side += 2;
                let mask = !(Bitboard::from(Rank::R1) | Rank::R8.into());
                let mut targets = self.targets & mask;
                if self.victim == Pawn {
                    targets |= self.ep_mask;
                };
                let pieces = pos.occupied_by_piece(pos.turn(), self.attacker);
                self.destinations = pieces.shift_xy(self.side, self.forward) & targets;
                self.pieces = Bitboard::new();
            } else if let Some(orig) = self.pieces.pop() {
                // switch to new attacking piece of same type
                self.orig = orig;
                self.destinations = self.targets & match self.attacker {
                    Pawn => unreachable!(),
                    Knight => knight_attacks(orig),
                    Bishop => bishop_attacks(orig, pos.occupied()),
                    Rook => rook_attacks(orig, pos.occupied()),
                    Queen => queen_attacks(orig, pos.occupied()),
                    King => king_attacks(orig),
                };
            } else if self.attacker < King {
                // switch to new attacking piece type
                self.attacker = match self.attacker {
                    Pawn => Knight,
                    Knight => Bishop,
                    Bishop => Rook,
                    Rook => Queen,
                    Queen => King,
                    King => unreachable!(),
                };
                self.pieces = pos.occupied_by_piece(pos.turn(), self.attacker);
            } else if self.victim > Pawn {
                // switch to new victim piece type
                self.victim = match self.victim {
                    Queen => Rook,
                    Rook => Bishop,
                    Bishop => Knight,
                    Knight => Pawn,
                    _ => unreachable!(),
                };
                self.targets = pos.occupied_by_piece(!pos.turn(), self.victim);
                self.attacker = Pawn;
                self.pieces = pos.occupied_by_piece(pos.turn(), self.attacker);
                self.side = -3;
            } else {
                self.state = UnderPromotions;
            }
        }

        // --- Under Promotions ---
        self.under_promotions.pop_front()
    }
}

impl<'a> FusedIterator for PromotionsAndCaptures<'a> { }
