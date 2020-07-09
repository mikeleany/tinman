//! Contains structures related to the `Position`.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::convert::TryInto;
use std::fmt;
use super::*;

use bitboard::*;

use Color::*;
use Piece::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A representation of the arrangement of pieces on the board at a given point in the game, as well
/// as castling availability and en passant legality.
///
/// # Instantiation
/// There are four typical ways of creating a new `Position` structure.
///  -  The [`new`](#method.new) method creates a `Position` structure containing the standard
///     starting position.
///  -  The [`from_fen_str`](#method.from_fen_str) method (along with its synonyms `from_str` and
///     `str::parse`) creates a new `Position` structure from a string containing [Forsyth-Edwards
///     Notation (FEN)](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
///  -  Using a [`PositionBuilder`](struct.PositionBuilder.html).
///  -  As a result of [`Move::make`](struct.Move.html#method.make).
///
/// # Generating Moves
/// The most important thing that can be done with a `Position` is to generate a list of legal
/// [`Move`](struct.Move.html)s from that `Position`. The [`moves`](#method.moves) method generates
/// all valid moves, while [`promotions_and_captures`](#method.promotions_and_captures) generates
/// only moves which gain material.
///
/// A typical flow might look something like this:
///
/// ```rust
/// use chess::Position;
/// use chess::ValidMove;
///
/// let pos = Position::new();
///
/// for mov in pos.moves() {
///     if let Ok(new_pos) = mov.make() {
///         // do something useful
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    zobrist: Zobrist,
    occ_squares: Bitboard,
    occ_by_color: [Bitboard; Color::COUNT],
    occ_by_piece: [[Bitboard; Piece::COUNT]; Color::COUNT],
    turn: Color,

    in_check: bool,
    ep_square: Option<Square>,
    castling_rights: [u8; Color::COUNT],

    draw_plies: usize,
    move_num: usize,
}


const CASTLE_KING_SIDE: u8 = 0x1;
const CASTLE_QUEEN_SIDE: u8 = 0x2;
const CASTLE_BOTH_SIDES: u8 = CASTLE_KING_SIDE | CASTLE_QUEEN_SIDE;

impl Position {

    /// Returns the standard starting Position.
    pub fn new() -> Position {
        let mut pos = Position {
            zobrist: Zobrist::new(),
            occ_squares: Bitboard::from(0xc3c3_c3c3_c3c3_c3c3u64),
            occ_by_color: [
                // white
                Bitboard::from(0x0303_0303_0303_0303u64),
                // black
                Bitboard::from(0xc0c0_c0c0_c0c0_c0c0u64)
            ],
            occ_by_piece:  [
                // white
                [
                    // pawns
                    Bitboard::from(0x0202_0202_0202_0202u64),
                    // knights
                    Bitboard::from(0x0001_0000_0000_0100u64),
                    // bishops
                    Bitboard::from(0x0000_0100_0001_0000u64),
                    // rooks
                    Bitboard::from(0x0100_0000_0000_0001u64),
                    // queen
                    Bitboard::from(0x0000_0000_0100_0000u64),
                    // king
                    Bitboard::from(0x0000_0001_0000_0000u64),
                ],
                // black
                [
                    // pawns
                    Bitboard::from(0x4040_4040_4040_4040u64),
                    // knights
                    Bitboard::from(0x0080_0000_0000_8000u64),
                    // bishops
                    Bitboard::from(0x0000_8000_0080_0000u64),
                    // rooks
                    Bitboard::from(0x8000_0000_0000_0080u64),
                    // queen
                    Bitboard::from(0x0000_0000_8000_0000u64),
                    // king
                    Bitboard::from(0x0000_0080_0000_0000u64),
                ],
            ],
            turn: White,
            in_check: false,
            ep_square: None,
            castling_rights: [CASTLE_BOTH_SIDES, CASTLE_BOTH_SIDES],
            draw_plies: 0,
            move_num: 1,
        };

        pos.calc_zobrist();

        pos
    }

    /// Returns a position with an empty board.
    fn empty_board() -> Position {
        Position{
            zobrist: Zobrist::new(),
            occ_squares: Bitboard::new(),
            occ_by_color: [Bitboard::new(); Color::COUNT],
            occ_by_piece: [[Bitboard::new(); Piece::COUNT]; Color::COUNT],
            turn: White,
            in_check: false,
            ep_square: None,
            castling_rights: [0, 0],
            draw_plies: 0,
            move_num: 1,
        }
    }

    /// Parse a position from a string containing [Forsyth-Edwards
    /// Notation (FEN)](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
    pub fn from_fen_str(s: &str) -> Result<Position> {
        use Error::*;

        let mut pos = PositionBuilder::new();
        let mut fields = s.trim().split_whitespace();

        // parse the board
        if let Some(board) = fields.next() {
            let mut r = Rank::COUNT - 1;
            let mut f = 0;
            for c in board.chars() {
                match c {
                    '1' ..= '8' => {
                        f += c.to_digit(10).expect("INFALLIBLE") as usize;
                        if f > 8 {
                            return Err(ParseError);
                        }
                    }
                    '/' => {
                        if f == File::COUNT && r > 0 {
                            r -= 1;
                            f = 0;
                        } else {
                            return Err(ParseError);
                        }
                    }
                    _ => {
                        let sq = match (f.try_into(), r.try_into()) {
                            (Ok(f), Ok(r)) => Square::from_coord(f, r),
                            _ => return Err(ParseError),
                        };
                        let color = if c.is_uppercase() { White } else { Black };
                        let piece: Piece = c.to_string().parse()?;

                        // set the `sq` as occupied
                        pos.piece(color, piece, sq);

                        f += 1;
                    }
                }
            }
            if r > 0 || f < 8 {
                return Err(ParseError);
            }
        } else {
            return Err(ParseError);
        }

        // parse the turn
        match fields.next() {
            Some(turn) => { pos.turn(turn.parse()?); },
            None => return Err(ParseError),
        }

        // parse the castling flags
        match fields.next() {
            Some("-") => {},
            Some(castling_flags) => {
                for c in castling_flags.chars() {
                    match c {
                        'K' => { pos.can_castle_king_side(White, true); },
                        'Q' => { pos.can_castle_queen_side(White, true); },
                        'k' => { pos.can_castle_king_side(Black, true); },
                        'q' => { pos.can_castle_queen_side(Black, true); },
                        _ => return Err(ParseError),
                    }
                }
            },
            None => return Err(ParseError),
        }

        // parse en passant square
        match fields.next() {
            Some("-") => {},
            Some(ep_square) => { pos.en_passant_square(Some(ep_square.parse()?)); },
            None => return Err(ParseError),
        }

        // parse half move clock, if present
        if let Some(plies) =  fields.next() {
            match plies.parse() {
                Ok(plies) => { pos.draw_plies(plies); },
                Err(_) => return Err(ParseError),
            }
        }

        // parse move number, if present
        if let Some(move_num) =  fields.next() {
            match move_num.parse() {
                Ok(move_num) => { pos.move_number(move_num); },
                Err(_) => return Err(ParseError),
            }
        }

        Ok(pos.validate()?)
    }

    /// Converts the position to a FEN string.
    pub fn to_fen_str(&self) -> String {
        // the board
        let mut board = String::new();
        let mut arr: [[Option<(Color, Piece)>;Rank::COUNT];File::COUNT]
            = [[None;Rank::COUNT];File::COUNT];

        for c in &[White, Black] {
            for p in &[Pawn, Knight, Bishop, Rook, Queen, King] {
                for sq in self.occupied_by_piece(*c, *p) {
                    arr[sq.file() as usize][sq.rank() as usize] = Some((*c, *p));
                }
            }
        }

        for r in (0..Rank::COUNT).rev() {
            let mut count = 0;
            for file in arr.iter() {
                if let Some((c, p)) = file[r] {
                    if count > 0 {
                        board += &count.to_string();
                        count = 0;
                    }

                    if c == White {
                        board += &p.to_string();
                    } else {
                        board += &p.to_string().to_lowercase();
                    }
                }
                else {
                    count += 1;
                }
            }
            if count > 0 {
                board += &count.to_string();
            }
            if r > 0 {
                board += "/";
            }
        }

        // whose turn it is
        let turn = self.turn.to_string();

        // castling rights
        let mut castling = String::new();
        castling += match self.castling_rights[White as usize] {
            CASTLE_KING_SIDE => "K",
            CASTLE_QUEEN_SIDE => "Q",
            CASTLE_BOTH_SIDES => "KQ",
            _ => "",
        };
        castling += match self.castling_rights[Black as usize] {
            CASTLE_KING_SIDE => "k",
            CASTLE_QUEEN_SIDE => "q",
            CASTLE_BOTH_SIDES => "kq",
            _ => "",
        };
        if castling == "" {
            castling += "-";
        }

        // en passant square
        let ep_square = match self.ep_square {
            Some(sq) => sq.to_string(),
            None => "-".to_string(),
        };

        // halfmove clock
        let half_move_clock = self.draw_plies.to_string();

        // fullmove number
        let full_move_number = self.move_num.to_string();

        // return the full fen string
        format!("{} {} {} {} {} {}", board, turn, castling, ep_square,
                                     half_move_clock, full_move_number)
    }

    /// Returns the color whose turn it is.
    pub fn turn(&self) -> Color {
        self.turn
    }

    /// Returns the en-passant square, if any.
    pub fn en_passant_square(&self) -> Option<Square> {
        self.ep_square
    }

    /// Returns `true` if the color to move is in check.
    pub fn in_check(&self) -> bool {
        self.in_check
    }

    /// Returns `true` if king-side castling rights are available for `c`.
    pub fn has_king_side_castling_rights(&self, c: Color) -> bool {
        self.castling_rights[c as usize] & CASTLE_KING_SIDE != 0
    }

    /// Returns `true` if queen-side castling rights are available for `c`.
    pub fn has_queen_side_castling_rights(&self, c: Color) -> bool {
        self.castling_rights[c as usize] & CASTLE_QUEEN_SIDE != 0
    }

    /// Returns `true` if any castling rights are available for `c`.
    pub fn has_castling_rights(&self, c: Color) -> bool {
        self.castling_rights[c as usize] != 0
    }

    /// Returns `true` if a draw by the fifty move rule can be claimed (assuming the game isn't
    /// already over).
    pub fn fifty_moves(&self) -> bool {
        self.draw_plies >= 100
    }

    /// Returns the number of plies which count toward the fifty move rule.
    pub fn draw_plies(&self) -> usize {
        self.draw_plies
    }

    /// Returns the move number.
    pub fn move_number(&self) -> usize {
        self.move_num
    }

    /// Returns a `Bitboard` of all occupied `Square`s.
    pub fn occupied(&self) -> Bitboard {
        self.occ_squares
    }

    /// Returns a `Bitboard` of `Squares` occupied by player `c`.
    pub fn occupied_by(&self, c: Color) -> Bitboard {
        self.occ_by_color[c as usize]
    }

    /// Returns a `Bitboard` of `Squares` occupied by the given `Piece` and `Color`.
    pub fn occupied_by_piece(&self, c: Color, p: Piece) -> Bitboard {
        self.occ_by_piece[c as usize][p as usize]
    }

    /// Returns the square where the king of the given color is located.
    pub fn king_location(&self, c: Color) -> Square {
        self.occupied_by_piece(c, King).peek().expect("INFALLIBLE")
    }

    /// Returns the color and type of piece, if any, at the given location.
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        if self.occ_squares.contains(sq) {
            for c in &[White, Black] {
                if self.occupied_by(*c).contains(sq) {
                    for p in &[Pawn, Knight, Bishop, Rook, Queen, King] {
                        if self.occupied_by_piece(*c, *p).contains(sq) {
                            return Some((*c, *p));
                        }
                    }
                    unreachable!()
                }
            }
            unreachable!()
        }

        None
    }

    /// Return the position's Zobrist key.
    pub fn zobrist_key(&self) -> Zobrist {
        self.zobrist
    }

    /// Returns `true` if there is insufficient material for checkmate.
    pub fn insufficient_material(&self) -> bool {
        if self.occ_squares.len() == 2 {
            return true;
        } else if self.occ_squares.len() == 3 {
            let bishops_and_knights =
                  self.occupied_by_piece(White, Knight)
                | self.occupied_by_piece(Black, Knight)
                | self.occupied_by_piece(White, Bishop)
                | self.occupied_by_piece(Black, Bishop);

            if !bishops_and_knights.is_empty() {
                return true;
            }
        }

        false
    }

    /// Returns an iterator over valid (pseudo-legal) moves from this position.
    ///
    /// Note that the iterator does not validate if the moves leave the mover in check or if they
    /// involve castling through check. Use [`Move::make()`](struct.Move.html#method.make) to
    /// perform those validations.
    pub fn moves(&self) -> Moves {
        Moves::new(self)
    }

    /// Returns an iterator over valid (pseudo-legal) promotions and captures from this position.
    ///
    /// Note that the iterator does not validate if the moves leave the mover in check or if they
    /// involve castling through check. Use [`Move::make()`](struct.Move.html#method.make) to
    /// perform those validations.
    pub fn promotions_and_captures(&self) -> PromotionsAndCaptures {
        PromotionsAndCaptures::new(self)
    }

    /// Make the move, returning the resulting position.
    pub fn make_move<T: ValidMove>(mv: &T) -> Result<Position> {
        if mv.move_type() == MoveType::NullMove {
            return mv.position().make_null_move();
        }

        let mut pos = mv.position().clone();

        // clear captured piece (including en passant)
        if let Some(capt_pc) = mv.captured_piece() {
            let sq = if mv.move_type() == MoveType::EnPassant {
                Square::from_coord(mv.destination().file(), mv.origin().rank())
            } else {
                mv.destination()
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
        let mask = Bitboard::from(mv.origin()) | mv.destination().into();
        pos.occ_squares ^= mask;
        pos.occ_by_color[pos.turn() as usize] ^= mask;
        pos.zobrist.toggle_piece_placement(pos.turn(), mv.piece(), mv.origin());
        match mv.move_type() {
            MoveType::Promotion(prom_pc) => {
                pos.occ_by_piece[pos.turn() as usize][mv.piece() as usize] ^= mv.origin().into();
                pos.occ_by_piece[pos.turn() as usize][prom_pc as usize] ^= mv.destination().into();
                pos.zobrist.toggle_piece_placement(pos.turn(), prom_pc.into(), mv.destination());
            },
            _ => {
                pos.occ_by_piece[pos.turn() as usize][mv.piece() as usize] ^= mask;
                pos.zobrist.toggle_piece_placement(pos.turn(), mv.piece(), mv.destination());
            },
        }

        // move rook for castling moves
        if mv.move_type() == MoveType::Castling {
            let rank = mv.origin().rank();
            let (orig, dest);
            match mv.destination().file() {
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
        let king_attacked = if mv.piece() != King && !pos.in_check() {
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
        if mv.move_type() == MoveType::Advance2 {
            pos.ep_square = match pos.turn() {
                White => Some(Square::from_coord(mv.destination().file(), Rank::R3)),
                Black => Some(Square::from_coord(mv.destination().file(), Rank::R6)),
            };
            pos.zobrist.toggle_ep_square(pos.en_passant_square().expect("INFALLIBLE"));
        } else {
            pos.ep_square = None;
        }

        // update castling rights if applicable
        match (pos.turn(), mv.origin()) {
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
        if mv.captured_piece().is_some() || mv.piece() == Pawn {
            pos.draw_plies = 0;
        } else {
            pos.draw_plies += 1;
        }

        // determine if opponent is now in check
        pos.in_check = match mv.piece() {
            Pawn | Knight => {
                pos.square_attacked_by(pos.king_location(pos.turn()), !pos.turn())
            },
            _ => {
                pos.square_attacked_by_sliding(pos.king_location(pos.turn()), !pos.turn())
            }
        };

        Ok(pos)
    }

    /// Make a null move. This is not a legal move, but can be useful to the chess engine.
    pub fn make_null_move(&self) -> Result<Position> {
        let mut pos = self.clone();

        // verify mover is not in check
        if pos.in_check() {
            return Err(Error::KingCapturable);
        }

        // update en passant square
        if let Some(ep_sq) = pos.en_passant_square() {
            pos.zobrist.toggle_ep_square(ep_sq);
            pos.ep_square = None;
        }

        // switch turns
        pos.turn = !pos.turn();
        pos.zobrist.toggle_turn();

        // update move counter
        if pos.turn() == White {
            pos.move_num += 1;
        }

        Ok(pos)
    }

    /// Calculate the `Positions`'s Zobrist key from scratch.
    fn calc_zobrist(&mut self) {
        self.zobrist = Zobrist::new();

        if self.turn == Black {
            self.zobrist.toggle_turn();
        }

        if let Some(ep_square) = self.ep_square {
            self.zobrist.toggle_ep_square(ep_square);
        }

        self.zobrist.toggle_castling_rights(White, self.castling_rights[White as usize]);
        self.zobrist.toggle_castling_rights(Black, self.castling_rights[Black as usize]);

        for c in &[ White, Black ] {
            for p in &[ Pawn, Knight, Bishop, Rook, Queen, King ] {
                for sq in self.occupied_by_piece(*c, *p) {
                    self.zobrist.toggle_piece_placement(*c, *p, sq);
                }
            }
        }
    }

    /// Returns `true` if `sq` is attacked by a piece of color `c`.
    pub fn square_attacked_by(&self, sq: Square, c: Color) -> bool {
        self.square_attacked_by_sliding(sq, c)
            || self.square_attacked_by_knight(sq, c)
            || self.square_attacked_by_king(sq, c)
            || self.pawn_attacks(c).contains(sq)
    }

    /// Returns `true` if `sq` is attacked by a sliding piece of color `c`.
    ///
    /// This is useful for finding discovered attacks.
    pub fn square_attacked_by_sliding(&self, sq: Square, c: Color) -> bool {
        let bishops = self.occupied_by_piece(c, Bishop);
        let rooks = self.occupied_by_piece(c, Rook);
        let queens = self.occupied_by_piece(c, Queen);

        bishop_attacks(sq, self.occ_squares).intersects(bishops | queens)
            || rook_attacks(sq, self.occ_squares).intersects(rooks | queens)
    }

    /// Returns `true` if `sq` is attacked by the king of color `c`.
    pub fn square_attacked_by_king(&self, sq: Square, c: Color) -> bool {
        king_attacks(sq).intersects(self.occupied_by_piece(c, King))
    }

    /// Returns `true` if `sq` is attacked by a knight of color `c`.
    pub fn square_attacked_by_knight(&self, sq: Square, c: Color) -> bool {
        knight_attacks(sq).intersects(self.occupied_by_piece(c, Knight))
    }

    /// Returns a bitboard containing all squares attacked by pawns of color `c`.
    pub fn pawn_attacks(&self, c: Color) -> Bitboard {
        let forward = if c == White { 1 } else { -1 };
        let pawns = self.occupied_by_piece(c, Pawn);
        pawns.shift_xy(-1, forward) | pawns.shift_xy(1, forward)
    }
}

impl Default for Position {
    /// Returns the standard starting Position.
    fn default() -> Self {
        Position::new()
    }
}

impl fmt::Display for Position {
    /// Writes out the position using FEN.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_fen_str().fmt(f)
    }
}

impl fmt::Debug for Position {
    /// Writes out the position using FEN.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_fen_str().fmt(f)
    }
}

impl FromStr for Position {
    type Err = Error;

    /// Parse a position from a FEN string.
    fn from_str(s: &str) -> Result<Self> {
        Position::from_fen_str(s)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod zobrist;
pub mod builder;
pub mod move_iter;

////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    /// Position::new() must return the standard starting position.
    ///
    /// Depends on to_fen_str() working properly.
    #[test]
    fn new_returns_the_standard_starting_position() {
        assert_eq!(Position::new().to_fen_str(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    /// Tests for Position::from_fen_str()
    mod from_fen_str {
        use super::*;
        use Error::*;

        // 1. empty string returns Err(ParseError)
        #[test]
        fn empty_string_returns_error() {
            assert_eq!(Position::from_fen_str(""), Err(ParseError));
            assert_eq!(Position::from_fen_str(" \t\r\n"), Err(ParseError));
        }

        // 2. 0 or 9 in board string returns Err(ParseError)
        #[test]
        fn invalid_empty_square_count_returns_error() {
            assert_eq!(Position::from_fen_str("0K1k5/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/9/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
        }

        // 3. 1 and 8 do not return an error (if used correctly)
        #[test]
        fn valid_empty_square_count_is_ok() {
            Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1").expect("valid fen");
        }

        // 4. A rank with more than 8 squares returns Err(ParseError)
        #[test]
        fn rank_too_long_returns_error() {
            assert_eq!(Position::from_fen_str("K1k6/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5b/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8B w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/b8 w - - 0 1"), Err(ParseError));
        }

        // 5. A rank with less than 8 squares returns Err(ParseError)
        #[test]
        fn rank_too_short_returns_error() {
            assert_eq!(Position::from_fen_str("K1k4/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k3b/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/6B w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/b6 w - - 0 1"), Err(ParseError));
        }

        // 6. Too many ranks returns an error
        #[test]
        fn too_many_ranks_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8/7R w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
        }

        // 7. Too few ranks returns an error
        #[test]
        fn too_few_ranks_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/7Q w - - 0 1"), Err(ParseError));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8 w - - 0 1"), Err(ParseError));
        }

        // 8. Pieces on files a and h do not return an error
        // 9. Pieces on ranks 1 and 8 do not return an error
        #[test]
        fn edge_files_and_ranks_ok() {
            Position::from_fen_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .expect("valid fen");
        }

        // 12. Missing turn field returns Err(ParseError)
        #[test]
        fn missing_turn_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8"), Err(ParseError));
        }

        // 13. 'w' and 'b' set the turn correctly
        #[test]
        fn turn_set_correctly() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1")
                .expect("valid fen").turn(), Color::White);
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 b - - 0 1")
                .expect("valid fen").turn(), Color::Black);
        }

        // 14. Anything other than 'w' and 'b' returns Err(ParseError)
        #[test]
        fn invalid_turn_color_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 x - - 0 1"), Err(ParseError));
        }

        // 15. Missing castling flags field returns Err(ParseError)
        #[test]
        fn missing_castling_flag_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w"), Err(ParseError));
        }

        // 16. Invalid castling flags returns Err(ParseError)
        #[test]
        fn invalid_castling_flag_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w x - 0 1"), Err(ParseError));
        }

        // 17. "-" castling flag returns leaves all castling flags empty
        #[test]
        fn empty_castling_flags_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1").expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], 0);
            assert_eq!(pos.castling_rights[Color::Black as usize], 0);
        }

        // 18. Any combination of "KQkq" sets the appropriate flags
        #[test]
        fn castling_flags_set_correctly() {
            let pos = Position::from_fen_str("r3k2r/8/8/8/8/8/8/R3K2R w Kk - 0 1")
                .expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], CASTLE_KING_SIDE);
            assert_eq!(pos.castling_rights[Color::Black as usize], CASTLE_KING_SIDE);

            let pos = Position::from_fen_str("r3k2r/8/8/8/8/8/8/R3K2R w Qq - 0 1")
                .expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], CASTLE_QUEEN_SIDE);
            assert_eq!(pos.castling_rights[Color::Black as usize], CASTLE_QUEEN_SIDE);

            let pos = Position::from_fen_str("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
                .expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], CASTLE_BOTH_SIDES);
            assert_eq!(pos.castling_rights[Color::Black as usize], CASTLE_BOTH_SIDES);

            let pos = Position::from_fen_str("r3k2r/8/8/8/8/8/8/R3K2R w KQ - 0 1")
                .expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], CASTLE_BOTH_SIDES);
            assert_eq!(pos.castling_rights[Color::Black as usize], 0);

            let pos = Position::from_fen_str("r3k2r/8/8/8/8/8/8/R3K2R w kq - 0 1")
                .expect("valid fen");
            assert_eq!(pos.castling_rights[Color::White as usize], 0);
            assert_eq!(pos.castling_rights[Color::Black as usize], CASTLE_BOTH_SIDES);
        }

        // 19. Missing en passant field returns Err(ParseError)
        #[test]
        fn missing_en_passant_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w -"), Err(ParseError));
        }

        // 20. "-" in the en passant field sets `ep_square` to `None`
        #[test]
        fn no_en_passant_square_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1").expect("valid fen");
            assert_eq!(pos.ep_square, None);
        }

        // 21. An en passant field that is not a square returns Err(ParseError)
        #[test]
        fn bad_en_passant_square_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - x 0 1"), Err(ParseError));
        }

        // 22. A valid en passant square sets `ep_square` to that square
        #[test]
        fn valid_en_passant_square_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/7p/8/8/8/8 w - h6 0 1").expect("valid fen");
            assert_eq!(pos.ep_square, Some(Square::H6));
        }

        // 23. Missing half-move clock field leaves it set to zero
        #[test]
        fn missing_halfmove_clock_field_defaults_zero() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - -").expect("valid fen");
            assert_eq!(pos.draw_plies, 0);
        }

        // 24. Non-integer half-move clock field returns appropriate error
        #[test]
        fn bad_halfmove_clock_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - x 1"),
                Err(ParseError));
        }

        // 25. Integer half-move clock field sets the value
        #[test]
        fn valid_halfmove_clock_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 500 1").expect("valid fen");
            assert_eq!(pos.draw_plies, 500);
        }

        // 26. Missing full-move number field leaves it set to one
        #[test]
        fn missing_fullmove_number_field_defaults_one() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - -").expect("valid fen");
            assert_eq!(pos.move_num, 1);
        }

        // 27. Non-integer full-move number field returns the appropriate error
        #[test]
        fn bad_fullmove_number_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 x"),
                Err(ParseError));
        }

        // 28. Integer full-move number field sets the value
        #[test]
        fn valid_fullmove_number_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 9999").expect("valid fen");
            assert_eq!(pos.move_num, 9999);
        }

        // 29a. two kings on one side returns Err(InvalidKingCount)
        #[test]
        fn multiple_kings_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/7K/8 w - - 0 1"), Err(InvalidKingCount));
        }

        // 29. No kings on one side returns Err(InvalidKingCount)
        #[test]
        fn missing_king_returns_error() {
            assert_eq!(Position::from_fen_str("K7/8/8/8/8/8/8/8 w - - 0 1"), Err(InvalidKingCount));
        }

        // 30. Pawns on Rank 1 or 8 returns Err(InvalidPawnRank)
        #[test]
        fn pawns_on_first_or_last_rank_returns_error() {
            assert_eq!(Position::from_fen_str("K1k4p/8/8/8/8/8/8/8 w - - 0 1"),
                Err(InvalidPawnRank));
            assert_eq!(Position::from_fen_str("K1k4P/8/8/8/8/8/8/8 w - - 0 1"),
                Err(InvalidPawnRank));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/p7 w - - 0 1"),
                Err(InvalidPawnRank));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/P7 w - - 0 1"),
                Err(InvalidPawnRank));
        }

        // 31. Attacked opponent king results in Err(KingCapturable)
        #[test]
        fn capturable_king_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/2R5 w - - 0 1"),
                Err(KingCapturable));
        }

        // 32. Piece in en passant square results in Err(EnPassantSquareOccupied)
        #[test]
        fn en_passant_square_occupied_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/7p/7p/8/8/8/8 w - h6 0 1"),
                Err(EnPassantSquareOccupied));
        }

        // 33. Missing en passant capture pawn results in Err(MissingEnPassantPawn)
        #[test]
        fn missing_en_passant_capture_pawn_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - h6 0 1"),
                Err(MissingEnPassantPawn));
        }

        // 34. For each player, if king is out of origin location with castling rights,
        //      returns Err(InvalidCastlingFlags)
        #[test]
        fn castling_priviledges_when_king_has_moved_returns_error() {
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/7K/R6R w K - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/7K/R6R w Q - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("r6r/7k/8/8/8/8/8/2K5 w k - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("r6r/7k/8/8/8/8/8/2K5 w q - 0 1"),
                Err(InvalidCastlingFlags));
        }

        // 35. For each player, if the king-side rook is out of place with king-side castling
        //      rights, returns Err(InvalidCastlingFlags)
        // 36. For each player, if the queen-side rook is out of place with queen-side castling
        //      rights, returns Err(InvalidCastlingFlags)
        #[test]
        fn castling_priviledges_when_rook_has_moved_returns_error() {
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/8/4K3 w K - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/8/4K3 w Q - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("4k3/8/8/8/8/8/8/2K5 w k - 0 1"),
                Err(InvalidCastlingFlags));
            assert_eq!(Position::from_fen_str("4k3/8/8/8/8/8/8/2K5 w q - 0 1"),
                Err(InvalidCastlingFlags));
        }


        // 37. If no errors, to_fen_str() returns the input fen string
        #[test]
        fn back_to_identical_fen() {
            assert_eq!(
                Position::from_fen_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .expect("valid fen").to_fen_str(),
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            );
        }

    }
}
