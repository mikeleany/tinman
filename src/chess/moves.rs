//! Contains structures to represent and generate moves
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::sync::Arc;
use super::*;
use bitboard::*;
use Piece::*;
use Color::*;

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
/// involve castling through check. Use `ValidMove::make()` to verify full legality.
pub trait ValidMove {
    /// Returns the position from which this move is valid.
    fn position(&self) -> &Position;
    /// Returns the piece to be moved.
    fn piece(&self) -> Piece;
    /// Returns the origin of the moved piece.
    fn origin(&self) -> Square;
    /// Returns the destination of the moved piece.
    fn destination(&self) -> Square;
    /// Returns the captured piece, if any.
    fn captured_piece(&self) -> Option<Piece>;
    /// Returns the type of move.
    fn move_type(&self) -> MoveType;

    /// Returns the color of the piece being moved.
    fn color(&self) -> Color {
        self.position().turn()
    }
    /// Returns `true` if the move is a capture.
    fn is_capture(&self) -> bool {
        self.captured_piece().is_some()
    }
    /// Returns `true` if the move is a promotion.
    fn is_promotion(&self) -> bool {
        if let MoveType::Promotion(_) = self.move_type() {
            true
        } else {
            false
        }
    }
    /// Returns the type of promotion, if any
    fn promotion(&self) -> Option<Promotion> {
        if let MoveType::Promotion(prom_pc) = self.move_type() {
            Some(prom_pc)
        } else {
            None
        }
    }
    /// Make the move, returning the resulting position.
    fn make(&self) -> Result<Position> where Self: std::marker::Sized {
        Position::make_move(self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A valid (pseudo-legal) move from a specific position.
///
/// Note that the move might not be fully legal, specifically, it may leave the mover in check or
/// involve castling through check. Use `Move::make()` to verify full legality.
///
/// Cannot outlive the position it is tied to.
///
/// See the [ValidMove](trait.ValidMove.html) trait for a list of methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move<'a> {
    pub (super) pos: &'a Position,
    pub (super) piece: Piece,
    pub (super) orig: Square,
    pub (super) dest: Square,
    pub (super) capt_pc: Option<Piece>,
    pub (super) move_type: MoveType,
}

impl<'a> ValidMove for Move<'a> {
    fn position(&self) -> &Position {
        self.pos
    }
    fn piece(&self) -> Piece {
        self.piece
    }
    fn origin(&self) -> Square {
        self.orig
    }
    fn destination(&self) -> Square {
        self.dest
    }
    fn captured_piece(&self) -> Option<Piece> {
        self.capt_pc
    }
    fn move_type(&self) -> MoveType {
        self.move_type
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
/// A valid (pseudo-legal) move from a specific position, with no lifetime restrictions. 
///
/// Note that the move might not be fully legal, specifically, it may leave the mover in check or
/// involve castling through check. Use `ArcMove::make()` to verify full legality.
///
/// See the [ValidMove](trait.ValidMove.html) trait for a list of methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcMove {
    pos: Arc<Position>,
    piece: Piece,
    orig: Square,
    dest: Square,
    capt_pc: Option<Piece>,
    move_type: MoveType,
}

impl ArcMove {
    /// Returns the position from which this move is valid as an `Arc<Position>`.
    pub fn position_arc(&self) -> &Arc<Position> {
        &self.pos
    }

    /// Make the move, returning the resulting position as an `Arc<Position>`.
    pub fn make_arc(&self) -> Result<Arc<Position>> {
        Ok(Arc::new(self.make()?))
    }
}

impl ValidMove for ArcMove {
    fn position(&self) -> &Position {
        &self.pos
    }
    fn piece(&self) -> Piece {
        self.piece
    }
    fn origin(&self) -> Square {
        self.orig
    }
    fn destination(&self) -> Square {
        self.dest
    }
    fn captured_piece(&self) -> Option<Piece> {
        self.capt_pc
    }
    fn move_type(&self) -> MoveType {
        self.move_type
    }
}

impl From<Move<'_>> for ArcMove {
    fn from(mv: Move<'_>) -> ArcMove {
        ArcMove {
            pos: Arc::new(mv.pos.clone()),
            piece: mv.piece,
            orig: mv.orig,
            dest: mv.dest,
            capt_pc: mv.capt_pc,
            move_type: mv.move_type,
        }
    }
}

impl fmt::Display for ArcMove {
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
/// use tinman::chess::{Position, MoveBuilder, ValidMove};
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
                Queen => { attacks = queen_attacks(dest, pos.occupied()); },
                Rook => { attacks = rook_attacks(dest, pos.occupied()); },
                Bishop => { attacks = bishop_attacks(dest, pos.occupied()); },
                Knight => { attacks = knight_attacks(dest); },
                Pawn => {
                    // TODO: handle two-square advancement
                    let forward = if pos.turn() == White { 1 } else { -1 };
                    let rank_mask = Bitboard::from(dest.rank()).shift_y(-forward);
                    let rank_mask2 = (rank_mask & !pos.occupied()).shift_y(-forward);
                    let rank_mask = rank_mask | rank_mask2;
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
                if let Some(ep_square) = pos.en_passant_square() {
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
                if !bishop_attacks(orig, pos.occupied()).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            Rook => {
                if !rook_attacks(orig, pos.occupied()).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            Queen => {
                if !queen_attacks(orig, pos.occupied()).contains(dest) {
                    return Err(Error::IllegalMove);
                }
            },
            King => {
                match (orig.file(), dest.file()) {
                    (File::E, File::G) => {
                        if pos.has_king_side_castling_rights(pos.turn())
                            && rank_attacks(orig, pos.occupied())
                            .intersects(File::H.into()) {
                            move_type = MoveType::Castling;
                        } else {
                            return Err(Error::IllegalMove);
                        }
                    },
                    (File::E, File::C) => {
                        if pos.has_queen_side_castling_rights(pos.turn())
                            && rank_attacks(orig, pos.occupied())
                            .intersects(File::A.into()) {
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
            }

            c = if let Some(c) = next {
                c.to_string()
            } else {
                // missing destination
                return Err(Error::ParseError);
            };
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

    #[test]
    fn bishop_to_c3() -> Result<(), crate::chess::Error> {
        use crate::chess::{Position, MoveBuilder, ValidMove, Piece};

        let pos = "r3k2r/p1ppqp2/Bn2pbp1/3PN3/4P3/2p4p/PPPB1PPP/R3K2R w KQkq - 0 3".parse()?;
        let mv = "Bc3".parse::<MoveBuilder>()?
            .validate(&pos)?;

        assert_eq!(mv.piece(), Piece::Bishop);

        Ok(())
    }

    #[test]
    fn validate_e4() -> Result<(), crate::chess::Error> {
        use crate::chess::{Position, MoveBuilder, ValidMove, Square};

        let pos = Position::default();
        let mv = "e4".parse::<MoveBuilder>()?
            .validate(&pos)?;

        assert_eq!(mv.to_string(), "e4".to_string());

        let pos = "r1bqkbnr/pppp1ppp/2n5/8/8/4PN2/PPP1PPPP/RNBQKB1R w KQkq - 1 5".parse()?;
        let mv = "e4".parse::<MoveBuilder>()?
            .validate(&pos)?;

        assert_eq!(mv.origin(), Square::E3);

        Ok(())
    }
}
