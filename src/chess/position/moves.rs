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

////////////////////////////////////////////////////////////////////////////////////////////////////
/// The type of move
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveType {
    /// Any move which is not castling, a two-square pawn advancement, en-passant capture, or pawn
    /// promotion
    Standard,
    /// A castling move
    Castling,
    /// A two-square pawn advancement
    Advance2,
    /// An en passant capture
    EnPassant,
    /// A pawn promotion to the given piece type
    Promotion(Promotion),
}

impl MoveType {
    /// Returns `true` if the `MoveType` is a promotion.
    pub fn is_promotion(self) -> bool {
        if let MoveType::Promotion(_) = self {
            true
        } else {
            false
        }
    }
}

impl Default for MoveType {
    fn default() -> Self {
        MoveType::Standard
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Which piece to promote to for a promotion move
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Promotion {
    ToKnight = 1,
    ToBishop = 2,
    ToRook = 3,
    ToQueen = 4,
}

use Promotion::*;

impl Default for Promotion {
    fn default() -> Self {
        ToQueen
    }
}

impl From<Promotion> for Piece {
    fn from(prom: Promotion) -> Self {
        unsafe { mem::transmute::<Promotion, Piece>(prom) }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A valid (pseudo-legal) move from a specific position.
///
/// Note that the move might not be fully legal, specifically, it may leave the mover in check or
/// involve castling through check. Use `Move::make()` to verify full legality.
///
/// Cannot outlive the position it is tied to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move<'a> {
    pub (super) pos: &'a Position,
    pub (super) piece: Piece,
    pub (super) orig: Square,
    pub (super) dest: Square,
    pub (super) capt_pc: Option<Piece>,
    pub (super) move_type: MoveType,
}

impl<'a> Move<'a> {
    /// Returns the position from which this move is valid.
    pub fn position(&self) -> &Position {
        self.pos
    }
    /// Returns the piece to be moved.
    pub fn piece(&self) -> Piece {
        self.piece
    }
    /// Returns the origin of the moved piece.
    pub fn origin(&self) -> Square {
        self.orig
    }
    /// Returns the destination of the moved piece.
    pub fn destination(&self) -> Square {
        self.dest
    }
    /// Returns the captured piece, if any.
    pub fn captured_piece(&self) -> Option<Piece> {
        self.capt_pc
    }
    /// Returns the type of promotion, if any
    pub fn promotion(&self) -> Option<Promotion> {
        if let MoveType::Promotion(prom_pc) = self.move_type {
            Some(prom_pc)
        } else {
            None
        }
    }
    /// Returns the type of move.
    pub fn move_type(&self) -> MoveType {
        self.move_type
    }
    /// Returns `true` if the move is a capture.
    pub fn is_capture(&self) -> bool {
        self.capt_pc.is_some()
    }
    /// Returns `true` if the move is a promotion.
    pub fn is_promotion(&self) -> bool {
        if let MoveType::Promotion(_) = self.move_type {
            true
        } else {
            false
        }
    }
    /// Make the move, returning the resulting position.
    pub fn make(&self) -> Result<Position> {
        let mut pos = self.pos.clone();

        // clear captured piece (including en passant)
        if let Some(capt_pc) = self.capt_pc {
            let sq = if self.move_type == MoveType::EnPassant {
                Square::from_coord(self.dest.file(), self.orig.rank())
            } else {
                self.dest
            };

            let mask: Bitboard = sq.into();
            pos.occ_squares ^= mask;
            pos.occ_by_color[!pos.turn() as usize] ^= mask;
            pos.occ_by_piece[!pos.turn() as usize][capt_pc as usize] ^= mask;
            pos.zobrist.toggle_piece_placement(!pos.turn(), capt_pc, sq);

            // update opponent's castling rights if applicable
            match (!pos.turn(), sq) {
                (White, Square::A1) | (Black, Square::A8) => {
                    if pos.has_queen_side_castling_rights(!pos.turn()) {
                        pos.castling_rights[!pos.turn() as usize] &= !CASTLE_QUEEN_SIDE;
                        pos.zobrist.toggle_castling_rights(!pos.turn(), CASTLE_QUEEN_SIDE);
                    }
                },
                (White, Square::H1) | (Black, Square::H8) => {
                    if pos.has_king_side_castling_rights(!pos.turn()) {
                        pos.castling_rights[!pos.turn() as usize] &= !CASTLE_KING_SIDE;
                        pos.zobrist.toggle_castling_rights(!pos.turn(), CASTLE_KING_SIDE);
                    }
                },
                _ => {},
            }
        }

        // move piece to new location (update piece type if promotion)
        let mask = Bitboard::from(self.orig) | self.dest.into();
        pos.occ_squares ^= mask;
        pos.occ_by_color[pos.turn() as usize] ^= mask;
        pos.zobrist.toggle_piece_placement(pos.turn(), self.piece, self.orig);
        match self.move_type {
            MoveType::Promotion(prom_pc) => {
                pos.occ_by_piece[pos.turn() as usize][self.piece as usize] ^= self.orig.into();
                pos.occ_by_piece[pos.turn() as usize][prom_pc as usize] ^= self.dest.into();
                pos.zobrist.toggle_piece_placement(pos.turn(), prom_pc.into(), self.dest);
            },
            _ => {
                pos.occ_by_piece[pos.turn() as usize][self.piece as usize] ^= mask;
                pos.zobrist.toggle_piece_placement(pos.turn(), self.piece, self.dest);
            },
        }

        // move rook for castling moves
        if self.move_type == MoveType::Castling {
            let rank = self.orig.rank();
            let (orig, dest);
            match self.dest.file() {
                File::C => {
                    orig = Square::from_coord(File::A, rank);
                    dest = Square::from_coord(File::D, rank);
                },
                File::G => {
                    orig = Square::from_coord(File::H, rank);
                    dest = Square::from_coord(File::F, rank);
                },
                _ => unreachable!(),
            }

            if pos.square_attacked_by(dest, !pos.turn()) {
                // castling through check
                return Err(Error::CastlingThroughCheck);
            }

            let mask = Bitboard::from(orig) | dest.into();
            pos.occ_squares ^= mask;
            pos.occ_by_color[pos.turn() as usize] ^= mask;
            pos.occ_by_piece[pos.turn() as usize][Rook as usize] ^= mask;
            pos.zobrist.toggle_piece_placement(pos.turn(), Rook, orig);
            pos.zobrist.toggle_piece_placement(pos.turn(), Rook, dest);
        }

        // verify mover is not in check
        let king_attacked = if self.piece != King && !pos.in_check() {
            pos.square_attacked_by_sliding(pos.king_location(pos.turn()), !pos.turn())
        } else {
            pos.square_attacked_by(pos.king_location(pos.turn()), !pos.turn())
        };
        if king_attacked {
            // own king is under attack
            return Err(Error::KingCapturable);
        }

        // update en passant square
        if let Some(ep_sq) = pos.en_passant_square() {
            pos.zobrist.toggle_ep_square(ep_sq);
        }
        if self.move_type == MoveType::Advance2 {
            pos.ep_square = match pos.turn() {
                White => Some(Square::from_coord(self.dest.file(), Rank::R3)),
                Black => Some(Square::from_coord(self.dest.file(), Rank::R6)),
            };
            pos.zobrist.toggle_ep_square(pos.en_passant_square().expect("INFALLIBLE"));
        } else {
            pos.ep_square = None;
        }

        // update castling rights if applicable
        match (pos.turn(), self.orig) {
            (White, Square::A1) | (Black, Square::A8) => {
                if pos.has_queen_side_castling_rights(pos.turn()) {
                    pos.castling_rights[pos.turn() as usize] &= !CASTLE_QUEEN_SIDE;
                    pos.zobrist.toggle_castling_rights(pos.turn(), CASTLE_QUEEN_SIDE);
                }
            },
            (White, Square::H1) | (Black, Square::H8) => {
                if pos.has_king_side_castling_rights(pos.turn()) {
                    pos.castling_rights[pos.turn() as usize] &= !CASTLE_KING_SIDE;
                    pos.zobrist.toggle_castling_rights(pos.turn(), CASTLE_KING_SIDE);
                }
            },
            (White, Square::E1) | (Black, Square::E8) => {
                if pos.has_castling_rights(pos.turn()) {
                    let castling_rights = pos.castling_rights[pos.turn() as usize];
                    pos.castling_rights[pos.turn() as usize] = 0;
                    pos.zobrist.toggle_castling_rights(pos.turn(), castling_rights);
                }
            },
            _ => {},
        }

        // switch turns
        pos.turn = !pos.turn();
        pos.zobrist.toggle_turn();

        // update move counters
        if pos.turn() == White {
            pos.move_num += 1;
        }
        if self.capt_pc.is_some() || self.piece == Pawn {
            pos.draw_plies = 0;
        } else {
            pos.draw_plies += 1;
        }

        // determine if opponent is now in check
        pos.in_check = match self.piece {
            Pawn | Knight => {
                pos.square_attacked_by(pos.king_location(pos.turn()), !pos.turn())
            },
            _ => {
                pos.square_attacked_by_sliding(pos.king_location(pos.turn()), !pos.turn())
            }
        };

        Ok(pos)
    }
}

impl<'a> fmt::Display for Move<'a> {
    /// The move is formatted as follows:
    ///
    /// "{}" -- Standard Algebraic Notation (eg Nf3, e8=Q, or O-O)
    ///
    /// "{:+}" -- Long Algebraic Notation (eg Ng1-f3, e7-e8=Q, or O-O)
    ///
    /// "{:#}" -- Coordinate Notation (eg g1f3, e7e8q, or e1g1)
    ///
    /// "{:+#}" -- Alternate Long Algebraic Notation (eg Ng1f3, e7e8Q, or Ke1g1)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.move_type == MoveType::Castling && !f.alternate() {
            match self.dest.file() {
                File::G => return "O-O".fmt(f),
                File::C => return "O-O-O".fmt(f),
                _ => unreachable!(),
            }
        }

        let mut s = String::new();

        if self.piece != Pawn && (!f.alternate() || f.sign_plus()) {
            s += &self.piece.to_string();
        }

        if f.alternate() || f.sign_plus() {
            s += &self.orig.to_string();
        } else if self.piece == Pawn {
            if self.capt_pc.is_some() {
                s += &self.orig.file().to_string();
            }
        } else {
            let all_pieces = self.pos.occupied_by_piece(self.pos.turn(), self.piece);
            let attacks = match self.piece {
                Pawn => unreachable!(),
                Knight => knight_attacks(self.dest),
                Bishop => bishop_attacks(self.dest, self.pos.occupied()),
                Rook => rook_attacks(self.dest, self.pos.occupied()),
                Queen => queen_attacks(self.dest, self.pos.occupied()),
                King => king_attacks(self.dest),
            };
            let eligible = all_pieces & attacks;

            if eligible != self.orig.into() {
                if eligible & self.orig.file().into() == self.orig.into() {
                    s += &self.orig.file().to_string()
                } else if eligible & self.orig.rank().into() == self.orig.into() {
                    s += &self.orig.rank().to_string()
                } else {
                    s += &self.orig.to_string();
                }
            }
        }

        if !f.alternate() {
            if self.capt_pc.is_some() {
                s += "x";
            } else if f.sign_plus() {
                s += "-";
            }
        }

        s += &self.dest.to_string();

        if let MoveType::Promotion(prom_pc) = self.move_type {
            if !f.alternate() {
                s += "=";
            }

            s += &Piece::from(prom_pc).to_string();
        }

        if f.alternate() && !f.sign_plus() {
            s.make_ascii_lowercase();
        }

        s.fmt(f)
    }
}

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

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A builder for `Move`
///
/// Below is how `MoveBuilder` might be used in combination with a hash.
///
/// ```rust
/// use tinman::chess::{Square, Promotion};
/// use tinman::chess::{Position, Move, MoveBuilder};
/// use tinman::chess::Result;
///
/// struct HashMove {
///     orig: Square,
///     dest: Square,
///     prom: Option<Promotion>,
/// }
///
/// fn get_hash_move(pos: &Position) -> Result<Move> {
///     let hash_move: HashMove = hash_entry(pos);
///     MoveBuilder::new()
///         .origin(hash_move.orig)
///         .destination(hash_move.dest)
///         .promotion(hash_move.prom)
///         .validate(pos)
/// }
///
/// # use tinman::chess::PositionBuilder;
/// # use tinman::chess::{Color, Piece};
/// #
/// # fn hash_entry(pos: &Position) -> HashMove {
/// #     match pos.piece_at(Square::A7) {
/// #         Some((Color::White, Piece::Pawn)) =>
/// #             HashMove { orig: Square::A7, dest: Square::A8, prom: Some(Promotion::ToQueen) },
/// #         _ => HashMove { orig: Square::G1, dest: Square::F3, prom: None },
/// #     }
/// # }
/// #
/// # let pos = PositionBuilder::new()
/// #     .piece(Color::White, Piece::King, Square::E1)
/// #     .piece(Color::White, Piece::Pawn, Square::A7)
/// #     .piece(Color::Black, Piece::King, Square::E8)
/// #     .turn(Color::White)
/// #     .validate()?;
/// # get_hash_move(&pos)?;
/// #
/// # let pos = Position::new();
/// # get_hash_move(&pos)?;
/// #
/// # Ok::<(), tinman::chess::Error>(())
/// ```
///
/// `MoveBuilder` can also be used to parse a `Move` from a string.
///
/// ```rust
/// use tinman::chess::{Position, MoveBuilder};
///
/// let pos = Position::new();
/// let move_str = "Nf3"; // string would usually come from a user
///
/// let new_pos = move_str.parse::<MoveBuilder>()?.validate(&pos)?.make()?;
/// # Ok::<(), tinman::chess::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveBuilder {
    piece: Option<Piece>,
    orig_file: Option<File>,
    orig_rank: Option<Rank>,
    dest: Option<Square>,
    prom_pc: Option<Promotion>,
    castle_dest: Option<File>,
}

impl<'a> MoveBuilder {
    /// Creates a new MoveBuilder
    pub fn new() -> Self {
        MoveBuilder {
            piece: None,
            orig_file: None,
            orig_rank: None,
            dest: None,
            prom_pc: None,
            castle_dest: None,
        }
    }

    /// Sets the piece
    pub fn piece(&mut self, piece: Piece) -> &mut Self {
        self.piece = Some(piece);
        self
    }

    /// Sets the origin
    pub fn origin(&mut self, orig: Square) -> &mut Self {
        self.orig_file = Some(orig.file());
        self.orig_rank = Some(orig.rank());
        self
    }

    /// Sets the origin file
    pub fn origin_file(&mut self, file: File) -> &mut Self {
        self.orig_file = Some(file);
        self
    }

    /// Sets the origin rank
    pub fn origin_rank(&mut self, rank: Rank) -> &mut Self {
        self.orig_rank = Some(rank);
        self
    }

    /// Sets the destination
    pub fn destination(&mut self, dest: Square) -> &mut Self {
        self.dest = Some(dest);
        self
    }

    /// Sets or clears the promotion piece
    pub fn promotion(&mut self, prom_pc: Option<Promotion>) -> &mut Self {
        self.prom_pc = prom_pc;
        self
    }

    /// Sets this as a king-side castling move for `turn`
    pub fn castle_king_side(&mut self) -> &mut Self {
        self.piece = Some(King);
        self.orig_file = Some(File::E);
        self.orig_rank = None;
        self.dest = None;
        self.castle_dest = Some(File::G);
        self.prom_pc = None;
        self
    }

    /// Sets this as a queen-side castling move for `turn`
    pub fn castle_queen_side(&mut self) -> &mut Self {
        self.piece = Some(King);
        self.orig_file = Some(File::E);
        self.orig_rank = None;
        self.dest = None;
        self.castle_dest = Some(File::C);
        self.prom_pc = None;
        self
    }

    /// Validates the pseudo-legality of the move, and returns a `Move` tied to `pos`
    ///
    /// Note that this function does not validate if the move leaves the mover in check or if it
    /// involves castling through check. Use `Move::make()` to perform those validations.
    pub fn validate(&self, pos: &'a Position) -> Result<Move<'a>> {
        let mut move_type = MoveType::Standard;

        // Step 1: Disambiguation
        let dest = if let Some(dest) = self.dest {
            dest
        } else if let Some(dest_file) = self.castle_dest {
            let rank = if pos.turn() == White { Rank::R1 } else { Rank::R8 };
            Square::from_coord(dest_file, rank)
        } else {
            return Err(Error::AmbiguousMove);
        };

        let orig;
        if let (Some(file), Some(rank)) = (self.orig_file, self.orig_rank) {
            orig = Square::from_coord(file, rank);
        } else if let (Some(file), None) = (self.orig_file, self.dest) {
            orig = Square::from_coord(file, dest.rank());
        } else {
            let mask: Bitboard = match (self.orig_file, self.orig_rank) {
                (Some(file), None) => file.into(),
                (None, Some(rank)) => rank.into(),
                _ => !Bitboard::new(),
            };

            let piece = if let Some(piece) = self.piece {
                piece
            } else {
                Pawn
            };

            let attacks;
            match piece {
                King => { attacks = king_attacks(dest); },
                Queen => { attacks = queen_attacks(dest, pos.occ_squares); },
                Rook => { attacks = rook_attacks(dest, pos.occ_squares); },
                Bishop => { attacks = bishop_attacks(dest, pos.occ_squares); },
                Knight => { attacks = knight_attacks(dest); },
                Pawn => {
                    let forward = if pos.turn() == White { 1 } else { -1 };
                    let rank_mask = Bitboard::from(dest.rank()).shift_y(-forward);
                    if let Some(file) = self.orig_file {
                        attacks = rank_mask & file.into();
                    } else {
                        attacks = rank_mask & dest.file().into();
                    }
                },
            };
            let mask = mask & pos.occupied_by_piece(pos.turn(), piece) & attacks;

            if mask.len() != 1 {
                return Err(Error::AmbiguousMove);
            }

            orig = mask.peek().expect("INFALLIBLE");
        }

        // Step 2: determine and validate move piece
        let piece = match pos.piece_at(orig) {
            Some((color, piece)) => {
                if self.piece.is_some() && self.piece != Some(piece) || color != pos.turn() {
                    return Err(Error::IllegalMove);
                }
                piece
            },
            None => return Err(Error::IllegalMove),
        };

        // Step 3: determine capture piece, if any, including en passant
        let capt_pc = match pos.piece_at(dest) {
            Some((color, capt_pc)) => {
                if color != pos.turn() {
                    Some(capt_pc)
                } else {
                    return Err(Error::IllegalMove);
                }
            },
            None => {
                if let Some(ep_square) = pos.ep_square {
                    if dest == ep_square && piece == Pawn {
                        move_type = MoveType::EnPassant;
                        Some(Pawn)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        };

        // Step 4: validate piece movement
        match piece {
            Pawn => {
                let (forward, initial) = if pos.turn() == White {
                    (1, Rank::R2)
                } else {
                    (-1, Rank::R7)
                };

                let rank_diff = (dest.rank() as i8 - orig.rank() as i8) * forward;
                let file_diff = dest.file() as i8 - orig.file() as i8;
                let file_diff = file_diff * file_diff;

                match (file_diff, rank_diff, capt_pc) {
                    (0, 2, None) if orig.rank() == initial => move_type = MoveType::Advance2,
                    (0, 1, None) | (1, 1, Some(_)) => {
                        // check for promotions
                        match dest.rank() {
                            Rank::R1 | Rank::R8 => {
                                if let Some(prom_pc) = self.prom_pc {
                                    move_type = MoveType::Promotion(prom_pc);
                                } else {
                                    move_type = MoveType::Promotion(Promotion::ToQueen);
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => return Err(Error::IllegalMove),
                }

            },
            Knight => {
                if !knight_attacks(orig).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            Bishop => {
                if !bishop_attacks(orig, pos.occ_squares).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            Rook => {
                if !rook_attacks(orig, pos.occ_squares).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            Queen => {
                if !queen_attacks(orig, pos.occ_squares).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            King => {
                match (orig.file(), dest.file()) {
                    (File::E, File::G) => {
                        if pos.has_king_side_castling_rights(pos.turn())
                            && rank_attacks(orig, pos.occ_squares)
                            .intersects(File::H.into()) {
                            move_type = MoveType::Castling;
                        } else {
                            return Err(Error::IllegalMove);
                        }
                    },
                    (File::E, File::C) => {
                        if pos.has_queen_side_castling_rights(pos.turn())
                            && rank_attacks(orig, pos.occ_squares)
                            .intersects(File::H.into()) {
                            move_type = MoveType::Castling;
                        } else {
                            return Err(Error::IllegalMove);
                        }
                    },
                    _ => {
                        if !king_attacks(orig).contains(dest) {
                            return Err(Error::IllegalMove);
                        }
                    }
                }
            },
        }

        // Step 5: validate promotions
        if self.prom_pc.is_some() && !move_type.is_promotion() {
            return Err(Error::IllegalMove);
        }

        Ok(Move{
            pos,
            piece,
            orig,
            dest,
            capt_pc,
            move_type,
        })
    }
}

impl FromStr for MoveBuilder {
    type Err = Error;

    fn from_str(s: &str) -> Result<MoveBuilder> {
        let mut builder = MoveBuilder::new();

        // handle PGN/SAN-style castling notation
        match s {
            "O-O" | "0-0" => {
                builder.castle_king_side();
                return Ok(builder);
            },
            "O-O-O" | "0-0-0" => {
                builder.castle_queen_side();
                return Ok(builder);
            },
            _ => {},
        }

        let mut chars = s.chars();

        let mut next = chars.next_back();
        let mut c = if let Some(c) = next {
            c.to_string()
        } else {
            // empty string
            return Err(Error::ParseError);
        };

        // promotion piece
        let prom_pc = match c.as_str() {
            "Q" | "q" => Some(Promotion::ToQueen),
            "R" | "r" => Some(Promotion::ToRook),
            "B" | "b" => Some(Promotion::ToBishop),
            "N" | "n" => Some(Promotion::ToKnight),
            _ => None, // let validate move determine move type
        };

        if prom_pc.is_some() {
            builder.promotion(prom_pc);

            next = chars.next_back();
            if next == Some('=') {
                next = chars.next_back();
                c = if let Some(c) = next {
                    c.to_string()
                } else {
                    // missing destination
                    return Err(Error::ParseError);
                };
            }
        }

        // destination
        let dest_rank = Rank::from_str(&c)?;

        next =  chars.next_back();
        c = if let Some(c) = next {
            c.to_string()
        } else {
            // missing destination file
            return Err(Error::ParseError);
        };

        let dest_file = File::from_str(&c)?;

        next = chars.next_back();
        if next == Some('-') || next == Some('x') {
            next = chars.next_back();
        }

        let dest = Square::from_coord(dest_file, dest_rank);
        builder.destination(dest);

        // origin
        if let Some(c) = next {
            if let Ok(rank) = Rank::from_str(&c.to_string()) {
                builder.origin_rank(rank);
                next = chars.next_back();
            }
        }
        if let Some(c) = next {
            if let Ok(file) = File::from_str(&c.to_string()) {
                builder.origin_file(file);
                next = chars.next_back();
            }
        }

        // piece
        if let Some(c) = next {
            if let Ok(piece) = Piece::from_str(&c.to_string()) {
                builder.piece(piece);
                next = chars.next_back();
            } else {
                // cannot determine piece
                return Err(Error::ParseError);
            }
        }

        if next.is_some() {
            // extra characters
            return Err(Error::ParseError);
        }

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn move_type() {
        use std::mem::transmute;
        use super::MoveType;
        use super::Promotion::{ToKnight, ToBishop, ToRook, ToQueen};

        println!("{:?} = {}", MoveType::Standard,
            unsafe { transmute::<MoveType, u8>(MoveType::Standard) });
        println!("{:?} = {}", MoveType::Castling,
            unsafe { transmute::<MoveType, u8>(MoveType::Castling) });
        println!("{:?} = {}", MoveType::Advance2,
            unsafe { transmute::<MoveType, u8>(MoveType::Advance2) });
        println!("{:?} = {}", MoveType::EnPassant,
            unsafe { transmute::<MoveType, u8>(MoveType::EnPassant) });
        println!("{:?} = {}", MoveType::Promotion(ToKnight),
            unsafe { transmute::<MoveType, u8>(MoveType::Promotion(ToKnight)) });
        println!("{:?} = {}", MoveType::Promotion(ToBishop),
            unsafe { transmute::<MoveType, u8>(MoveType::Promotion(ToBishop)) });
        println!("{:?} = {}", MoveType::Promotion(ToRook),
            unsafe { transmute::<MoveType, u8>(MoveType::Promotion(ToRook)) });
        println!("{:?} = {}", MoveType::Promotion(ToQueen),
            unsafe { transmute::<MoveType, u8>(MoveType::Promotion(ToQueen)) });
    }
}
