//! Contains structures related to the `Position`
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
#[allow(missing_copy_implementations,)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Returns the standard starting Position
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

    /// Returns a position with an empty board
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

    /// Parse a position from a FEN string
    pub fn from_fen_str(s: &str) -> Result<Position, ParseFenError> {
        use ParseFenError::*;

        let mut pos = Position::empty_board();
        let mut fields = s.trim().split_whitespace();

        // parse the board
        if let Some(board) = fields.next() {
            let mut r = Rank::COUNT - 1;
            let mut f = 0;
            for c in board.chars() {
                match c {
                    '1' ... '8' => {
                        f += c.to_digit(10).expect("INFALLIBLE") as usize;
                        if f > 8 {
                            return Err(ParseBoard);
                        }
                    }
                    '/' => {
                        if f == File::COUNT && r > 0 {
                            r -= 1;
                            f = 0;
                        } else {
                            return Err(ParseBoard);
                        }
                    }
                    _ => {
                        let sq = match (f.try_into(), r.try_into()) {
                            (Ok(f), Ok(r)) => Square::from_coord(f, r),
                            _ => return Err(ParseBoard),
                        };
                        let color = if c.is_uppercase() { White } else { Black };
                        let piece: Piece = c.to_string().parse()?;

                        // set the `sq` as occupied
                        pos.occ_squares.insert(sq);
                        pos.occ_by_color[color as usize].insert(sq);
                        pos.occ_by_piece[color as usize][piece as usize].insert(sq);

                        f += 1;
                    }
                }
            }
            if r > 0 || f < 8 {
                return Err(ParseBoard);
            }
        } else {
            return Err(Empty);
        }

        // parse the turn
        match fields.next() {
            Some(turn) => pos.turn = turn.parse()?,
            None => return Err(ParseTurn),
        }

        // parse the castling flags
        match fields.next() {
            Some("-") => {},
            Some(castling_flags) => {
                for c in castling_flags.chars() {
                    match c {
                        'K' => pos.castling_rights[White as usize] |= CASTLE_KING_SIDE,
                        'Q' => pos.castling_rights[White as usize] |= CASTLE_QUEEN_SIDE,
                        'k' => pos.castling_rights[Black as usize] |= CASTLE_KING_SIDE,
                        'q' => pos.castling_rights[Black as usize] |= CASTLE_QUEEN_SIDE,
                        _ => return Err(ParseCastling),
                    }
                }
            },
            None => return Err(ParseCastling),
        }

        // parse en passant square
        match fields.next() {
            Some("-") => {},
            Some(ep_square) => pos.ep_square = Some(ep_square.parse()?),
            None => return Err(ParseEnPassant),
        }

        // parse half move clock, if present
        if let Some(plies) =  fields.next() {
            match plies.parse() {
                Ok(plies) => pos.draw_plies = plies,
                Err(_) => return Err(ParseHalfMoveClock),
            }
        }

        // parse move number, if present
        if let Some(move_num) =  fields.next() {
            match move_num.parse() {
                Ok(move_num) => pos.move_num = move_num,
                Err(_) => return Err(ParseMoveNumber),
            }
        }

        // validate position legality
        for c in 0..Color::COUNT {
            // Step 1: verify exactly one king per side
            if pos.occ_by_piece[c][King as usize].len() != 1 {
                return Err(KingCount);
            }
            // Step 2: no pawns on ranks 1 and 8
            if pos.occ_by_piece[c][Pawn as usize]
                .intersects(Bitboard::from(Rank::R1) | Rank::R8.into()) {
                return Err(InvalidPawnRank);
            }
        }
        // Step 3: opponent's king is not attacked
        if pos.square_attacked_by(pos.king_location(!pos.turn), pos.turn) {
            return Err(KingCapturable);
        }
        // Step 4: if there is an EP square, it must be empty and there must be a pawn to capture
        if let Some(ep_square) = pos.ep_square {
            if pos.piece_at(ep_square).is_some() {
                return Err(EnPassantPawn);
            }
            let forward = if pos.turn == White { 1 } else { -1 };
            if !pos.occ_by_piece[!pos.turn as usize][Pawn as usize]
                .shift_y(forward).contains(ep_square) {
                return Err(EnPassantPawn);
            }
        }
        // Step 5: if castling rights exist, king and rook must be in the correct squares
        for c in 0..Color::COUNT {
            if pos.castling_rights[c] != 0 {
                let r = if c == White as usize { Rank::R1 } else { Rank::R8 };

                if !pos.occ_by_piece[c][King as usize].contains(Square::from_coord(File::E, r)) {
                    return Err(InvalidCastling);
                }
                if pos.castling_rights[c] & CASTLE_QUEEN_SIDE != 0
                    && !pos.occ_by_piece[c][Rook as usize]
                        .contains(Square::from_coord(File::A, r)) {
                    return Err(InvalidCastling);
                }
                if pos.castling_rights[c] & CASTLE_KING_SIDE != 0
                    && !pos.occ_by_piece[c][Rook as usize]
                        .contains(Square::from_coord(File::H, r)) {
                    return Err(InvalidCastling);
                }
            }
        }

        pos.in_check = pos.square_attacked_by(pos.king_location(pos.turn), !pos.turn);
        pos.calc_zobrist();

        Ok(pos)
    }

    /// Converts the position to a FEN string
    pub fn to_fen_str(&self) -> String {
        // the board
        let mut board = String::new();
        let mut arr: [[Option<(Color, Piece)>;Rank::COUNT];File::COUNT]
            = [[None;Rank::COUNT];File::COUNT];

        for c in 0..Color::COUNT {
            let color: Color = c.try_into().expect("INVALLIBLE");
            for p in 0..Piece::COUNT {
                let piece: Piece = p.try_into().expect("INVALLIBLE");
                for sq in self.occ_by_piece[c][p] {
                    arr[sq.file() as usize][sq.rank() as usize] = Some((color, piece));
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

    /// Returns the color whose turn it is
    pub fn turn(&self) -> Color {
        self.turn
    }

    /// Returns `true` if the color to move is in check.
    pub fn in_check(&self) -> bool {
        self.in_check
    }

    /// Returns `true` if a draw by the fifty move rule can be claimed (assuming the game isn't
    /// already over)
    pub fn fifty_moves(&self) -> bool {
        self.draw_plies >= 100
    }

    /// Returns the number of plies which count toward the fifty move rule
    pub fn draw_plies(&self) -> usize {
        self.draw_plies
    }

    /// Returns the square where the king of the given color is located
    pub fn king_location(&self, c: Color) -> Square {
        self.occ_by_piece[c as usize][King as usize].peek().expect("INFALLIBLE")
    }

    /// Returns the color and type of piece, if any, at the given location
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        if self.occ_squares.contains(sq) {
            for c in 0..Color::COUNT {
                if self.occ_by_color[c].contains(sq) {
                    for p in 0..Piece::COUNT {
                        if self.occ_by_piece[c][p].contains(sq) {
                            return Some((c.try_into().expect("INFALLIBLE"),
                                        p.try_into().expect("INFALLIBLE")))
                        }
                    }
                    unreachable!()
                }
            }
            unreachable!()
        }

        None
    }

    /// Return the position's Zobrist key
    pub fn zobrist_key(&self) -> Zobrist {
        self.zobrist
    }

    /// Validates the pseudo-legality of the move from `orig` to `dest`, with the given move type,
    /// and returns a `Move` tied to this position.
    ///
    /// Note that this function does not validate if the move leaves the mover in check or if it
    /// involves castling through check. Use `Move::make()` to perform those validations.
    pub fn validate_move(&self,
        orig: Square,
        dest: Square,
        move_type: MoveType,
    ) -> Result<Move, ValidateMoveError> {
        let mut valid_move_type = MoveType::Standard;

        // Step 1: determine move piece and validate piece of correct color at orig
        let piece = match self.piece_at(orig) {
            Some((c, p)) => {
                if c != self.turn {
                    return Err(ValidateMoveError);
                }
                p
            },
            None => return Err(ValidateMoveError),
        };

        // Step 2: determine capture piece, if any, including en passant
        let capt_pc = match self.piece_at(dest) {
            Some((c, p)) => {
                if c != self.turn {
                    Some(p)
                } else {
                    return Err(ValidateMoveError);
                }
            },
            None => {
                if let Some(ep_square) = self.ep_square {
                    if dest == ep_square && piece == Pawn {
                        valid_move_type = MoveType::EnPassant;
                        Some(Pawn)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        };

        // Step 3: validate piece movement
        match piece {
            Pawn => {
                let (forward, initial) = if self.turn == White {
                    (1, Rank::R2)
                } else {
                    (-1, Rank::R7)
                };

                if capt_pc.is_some() {
                    // captures
                    let pc_board = Bitboard::from(orig);
                    let attacks = pc_board.shift_xy(-1, forward) | pc_board.shift_xy(1, forward);
                    if !attacks.contains(dest) {
                        return Err(ValidateMoveError);
                    }
                }
                else {
                    // advancement
                    if orig.file() != dest.file() {
                        return Err(ValidateMoveError);
                    }
                    let rank_diff = (dest.rank() as i8 - orig.rank() as i8) * forward;
                    match rank_diff {
                        2 if orig.rank() == initial => valid_move_type = MoveType::Advance2,
                        1 => {},
                        _ => return Err(ValidateMoveError),
                    }
                }

                // promotions
                match dest.rank() {
                    Rank::R1 | Rank::R8 => {
                        if let MoveType::Promotion(_) = move_type {
                            valid_move_type = move_type;
                        } else {
                            valid_move_type = MoveType::Promotion(Promotion::ToQueen);
                        }
                    },
                    _ => {},
                }
            },
            Knight => {
                if !knight_attacks(orig).contains(dest) {
                    return Err(ValidateMoveError);
                }
            },
            Bishop => {
                if !bishop_attacks(orig, self.occ_squares).contains(dest) {
                    return Err(ValidateMoveError);
                }
            },
            Rook => {
                if !rook_attacks(orig, self.occ_squares).contains(dest) {
                    return Err(ValidateMoveError);
                }
            },
            Queen => {
                if !queen_attacks(orig, self.occ_squares).contains(dest) {
                    return Err(ValidateMoveError);
                }
            },
            King => {
                match (orig.file(), dest.file()) {
                    (File::E, File::G) => {
                        if self.castling_rights[self.turn as usize] & CASTLE_KING_SIDE != 0
                            && rank_attacks(orig, self.occ_squares)
                            .intersects(File::H.into()) {
                            valid_move_type = MoveType::Castling;
                        } else {
                            return Err(ValidateMoveError);
                        }
                    },
                    (File::E, File::C) => {
                        if self.castling_rights[self.turn as usize] & CASTLE_QUEEN_SIDE != 0
                            && rank_attacks(orig, self.occ_squares)
                            .intersects(File::H.into()) {
                            valid_move_type = MoveType::Castling;
                        } else {
                            return Err(ValidateMoveError);
                        }
                    },
                    _ => {
                        if !king_attacks(orig).contains(dest) {
                            return Err(ValidateMoveError);
                        }
                    }
                }
            },
        }

        // Step 4: validate move type (allowing Standard to be used as unknown)
        if move_type != valid_move_type && move_type != MoveType::Standard {
            return Err(ValidateMoveError);
        }

        Ok(Move{
            pos: self,
            piece,
            orig,
            dest,
            capt_pc,
            move_type: valid_move_type,
        })
    }

    /// Parses a move from a string, validates the pseudo-legality of the move, and returns a `Move`
    /// tied to this position.
    ///
    /// Note that this function does not validate if the move leaves the mover in check or if it
    /// involves castling through check. Use `Move::make()` to perform those validations.
    pub fn move_from_str(&self, s: &str) -> Result<Move, ParseMoveError> {
        // [nbrqkNBRQK]?[a-h]?[1-8]?[-x]?[a-h][1-8]=?[nbrqkNBRQK]?
        let mut chars = s.chars();

        let mut next =  chars.next_back();
        let mut c = if let Some(c) = next {
            c.to_string()
        } else {
            // empty string
            return Err(ParseMoveError::ParseError);
        };

        // promotion piece
        let move_type = match c.as_str() {
            "Q" | "q" => MoveType::Promotion(Promotion::ToQueen),
            "R" | "r" => MoveType::Promotion(Promotion::ToRook),
            "B" | "b" => MoveType::Promotion(Promotion::ToBishop),
            "N" | "n" => MoveType::Promotion(Promotion::ToKnight),
            _ => MoveType::Standard, // let validate move determine move type
        };

        if let MoveType::Promotion(_) = move_type {
            next = chars.next_back();
            if next == Some('=') {
                next = chars.next_back();
                c = if let Some(c) = next {
                    c.to_string()
                } else {
                    // missing destination
                    return Err(ParseMoveError::ParseError);
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
            return Err(ParseMoveError::ParseError);
        };

        let dest_file = File::from_str(&c)?;

        next = chars.next_back();
        if next == Some('-') || next == Some('x') {
            next = chars.next_back();
        }

        let dest = Square::from_coord(dest_file, dest_rank);

        // origin
        let orig_rank = if let Some(c) = next {
            if let Ok(rank) = Rank::from_str(&c.to_string()) {
                next = chars.next_back();
                Some(rank)
            } else {
                None
            }
        } else {
            None
        };

        let orig_file = if let Some(c) = next {
            if let Ok(file) = File::from_str(&c.to_string()) {
                next = chars.next_back();
                Some(file)
            } else {
                None
            }
        } else {
            None
        };

        // piece
        let piece = if let Some(c) = next {
            if let Ok(pc) = Piece::from_str(&c.to_string()) {
                next = chars.next_back();
                Some(pc)
            } else {
                // cannot determine piece
                return Err(ParseMoveError::ParseError);
            }
        } else {
            None
        };

        if next.is_some() {
            // extra characters
            return Err(ParseMoveError::ParseError);
        }

        // disambiguation
        let orig;
        if let (Some(file), Some(rank)) = (orig_file, orig_rank) {
            orig = Square::from_coord(file, rank)
        } else {
            let mask: Bitboard = match (orig_file, orig_rank) {
                (Some(file), None) => file.into(),
                (None, Some(rank)) => rank.into(),
                _ => !Bitboard::new(),
            };

            let piece = if let Some(piece) = piece {
                piece
            } else {
                Pawn
            };

            let mask = mask & self.occ_by_piece[self.turn as usize][piece as usize] & match piece {
                King => { king_attacks(dest) },
                Queen => { queen_attacks(dest, self.occ_squares) },
                Rook => { rook_attacks(dest, self.occ_squares) },
                Bishop => { bishop_attacks(dest, self.occ_squares) },
                Knight => { knight_attacks(dest) },
                Pawn => {
                    let forward = if self.turn == White { 1 } else { -1 };
                    let rank_mask = Bitboard::from(dest_rank).shift_y(-forward);
                    if let Some(file) = orig_file {
                        rank_mask & file.into()
                    } else {
                        rank_mask & dest_file.into()
                    }
                },
            };

            if mask.len() != 1 {
                // ambiguous
                return Err(ParseMoveError::AmbiguousMove);
            }

            orig = mask.peek().expect("INFALLIBLE")
        };

        if let Some(piece) = piece {
            if self.piece_at(orig) != Some((self.turn, piece)) {
                // piece not at orig
                return Err(ParseMoveError::IllegalMove);
            }
        }

        Ok(self.validate_move(orig, dest, move_type)?)
    }

    /// Returns an iterator over valid (pseudo-legal) moves from this position.
    ///
    /// Note that the iterator does not validate if the moves leave the mover in check or if they
    /// involves castling through check. Use `Move::make()` to perform those validations.
    pub fn moves(&self) -> Moves {
        Moves::new(self)
    }

    /// Returns an iterator over valid (pseudo-legal) promotions and captures from this position.
    ///
    /// Note that the iterator does not validate if the moves leave the mover in check or if they
    /// involves castling through check. Use `Move::make()` to perform those validations.
    pub fn promotions_and_captures(&self) -> PromotionsAndCaptures {
        PromotionsAndCaptures::new(self)
    }

    /// Calculate the positions's Zobrist key from scratch
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
                for sq in self.occ_by_piece[*c as usize][*p as usize] {
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
        let bishops = self.occ_by_piece[c as usize][Bishop as usize];
        let rooks = self.occ_by_piece[c as usize][Rook as usize];
        let queens = self.occ_by_piece[c as usize][Queen as usize];

        bishop_attacks(sq, self.occ_squares).intersects(bishops | queens)
            || rook_attacks(sq, self.occ_squares).intersects(rooks | queens)
    }

    /// Returns `true` if `sq` is attacked by the king of color `c`.
    pub fn square_attacked_by_king(&self, sq: Square, c: Color) -> bool {
        king_attacks(sq).intersects(self.occ_by_piece[c as usize][King as usize])
    }

    /// Returns `true` if `sq` is attacked by a knight of color `c`.
    pub fn square_attacked_by_knight(&self, sq: Square, c: Color) -> bool {
        knight_attacks(sq).intersects(self.occ_by_piece[c as usize][Knight as usize])
    }

    /// Returns a bitboard containing all squares attacked by pawns of color `c`
    pub fn pawn_attacks(&self, c: Color) -> Bitboard {
        let forward = if c == White { 1 } else { -1 };
        let pawns = self.occ_by_piece[c as usize][Pawn as usize];
        pawns.shift_xy(-1, forward) | pawns.shift_xy(1, forward)
    }
}

impl Default for Position {
    /// Returns the standard starting Position
    fn default() -> Self {
        Position::new()
    }
}

impl fmt::Display for Position {
    /// Writes out the position using FEN
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_fen_str().fmt(f)
    }
}

impl FromStr for Position {
    type Err = ParseFenError;

    /// Parse a position from a FEN string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Position::from_fen_str(s)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod zobrist;
pub mod moves;

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

    /// Nothing to test for Position::empty_board()
    #[test]
    fn empty_board() {
        Position::empty_board();
    }

    /// Tests for Position::from_fen_str()
    mod from_fen_str {
        use super::*;
        use ParseFenError::*;

        // 1. empty string returns Err(Empty)
        #[test]
        fn empty_string_returns_error() {
            assert_eq!(Position::from_fen_str(""), Err(Empty));
            assert_eq!(Position::from_fen_str(" \t\r\n"), Err(Empty));
        }

        // 2. 0 or 9 in board string returns Err(ParseBoard)
        #[test]
        fn invalid_empty_square_count_returns_error() {
            assert_eq!(Position::from_fen_str("0K1k5/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/9/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
        }

        // 3. 1 and 8 do not return an error (if used correctly)
        #[test]
        fn valid_empty_square_count_is_ok() {
            Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1").expect("valid fen");
        }

        // 4. A rank with more than 8 squares returns Err(ParseBoard)
        #[test]
        fn rank_too_long_returns_error() {
            assert_eq!(Position::from_fen_str("K1k6/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5b/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8B w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/b8 w - - 0 1"), Err(ParseBoard));
        }

        // 5. A rank with less than 8 squares returns Err(ParseBoard)
        #[test]
        fn rank_too_short_returns_error() {
            assert_eq!(Position::from_fen_str("K1k4/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k3b/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/6B w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/b6 w - - 0 1"), Err(ParseBoard));
        }

        // 6. Too many ranks returns an error
        #[test]
        fn too_many_ranks_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8/7R w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
        }

        // 7. Too few ranks returns an error
        #[test]
        fn too_few_ranks_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/7Q w - - 0 1"), Err(ParseBoard));
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8 w - - 0 1"), Err(ParseBoard));
        }

        // 8. Pieces on files a and h do not return an error
        // 9. Pieces on ranks 1 and 8 do not return an error
        #[test]
        fn edge_files_and_ranks_ok() {
            Position::from_fen_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .expect("valid fen");
        }

        // 12. Missing turn field returns Err(ParseTurn)
        #[test]
        fn missing_turn_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8"), Err(ParseTurn));
        }

        // 13. 'w' and 'b' set the turn correctly
        #[test]
        fn turn_set_correctly() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1")
                .expect("valid fen").turn(), Color::White);
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 b - - 0 1")
                .expect("valid fen").turn(), Color::Black);
        }

        // 14. Anything other than 'w' and 'b' returns Err(ParseTurn)
        #[test]
        fn invalid_turn_color_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 x - - 0 1"), Err(ParseTurn));
        }

        // 15. Missing castling flags field returns Err(ParseCastling)
        #[test]
        fn missing_castling_flag_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w"), Err(ParseCastling));
        }

        // 16. Invalid castling flags returns Err(ParseCastling)
        #[test]
        fn invalid_castling_flag_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w x - 0 1"), Err(ParseCastling));
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

        // 19. Missing en passant field returns Err(ParseEnPassant)
        #[test]
        fn missing_en_passant_field_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w -"), Err(ParseEnPassant));
        }

        // 20. "-" in the en passant field sets `ep_square` to `None`
        #[test]
        fn no_en_passant_square_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 1").expect("valid fen");
            assert_eq!(pos.ep_square, None);
        }

        // 21. An en passant field that is not a square returns Err(ParseEnPassant)
        #[test]
        fn bad_en_passant_square_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - x 0 1"), Err(ParseEnPassant));
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
                Err(ParseHalfMoveClock));
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
                Err(ParseMoveNumber));
        }

        // 28. Integer full-move number field sets the value
        #[test]
        fn valid_fullmove_number_set_correctly() {
            let pos = Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - - 0 9999").expect("valid fen");
            assert_eq!(pos.move_num, 9999);
        }

        // 29a. two kings on one side returns Err(KingCount)
        #[test]
        fn multiple_kings_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/7K/8 w - - 0 1"), Err(KingCount));
        }

        // 29. No kings on one side returns Err(KingCount)
        #[test]
        fn missing_king_returns_error() {
            assert_eq!(Position::from_fen_str("K7/8/8/8/8/8/8/8 w - - 0 1"), Err(KingCount));
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

        // 32. Piece in en passant square results in Err(EnPassantPawn)
        #[test]
        fn en_passant_square_occupied_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/7p/7p/8/8/8/8 w - h6 0 1"),
                Err(EnPassantPawn));
        }

        // 33. Missing en passant capture pawn results in Err(EnPassantPawn)
        #[test]
        fn missing_en_passant_capture_pawn_returns_error() {
            assert_eq!(Position::from_fen_str("K1k5/8/8/8/8/8/8/8 w - h6 0 1"),
                Err(EnPassantPawn));
        }

        // 34. For each player, if king is out of origin location with castling rights,
        //      returns Err(InvalidCastling)
        #[test]
        fn castling_priviledges_when_king_has_moved_returns_error() {
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/7K/R6R w K - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/7K/R6R w Q - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("r6r/7k/8/8/8/8/8/2K5 w k - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("r6r/7k/8/8/8/8/8/2K5 w q - 0 1"),
                Err(InvalidCastling));
        }

        // 35. For each player, if the king-side rook is out of place with king-side castling
        //      rights, returns Err(InvalidCastling)
        // 36. For each player, if the queen-side rook is out of place with queen-side castling
        //      rights, returns Err(InvalidCastling)
        #[test]
        fn castling_priviledges_when_rook_has_moved_returns_error() {
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/8/4K3 w K - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("2k5/8/8/8/8/8/8/4K3 w Q - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("4k3/8/8/8/8/8/8/2K5 w k - 0 1"),
                Err(InvalidCastling));
            assert_eq!(Position::from_fen_str("4k3/8/8/8/8/8/8/2K5 w q - 0 1"),
                Err(InvalidCastling));
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
