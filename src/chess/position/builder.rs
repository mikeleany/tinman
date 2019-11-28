//! Contains a builder for `Position`
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use super::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A builder for `Position`
#[derive(Clone)]
#[allow(missing_debug_implementations)] // TODO: derive debug when it becomes possible
pub struct PositionBuilder {
    board: [ Option<(Color, Piece)>; Square::COUNT ],
    turn: Color,
    castle_king_side: [ bool; Color::COUNT ],
    castle_queen_side: [ bool; Color::COUNT ],
    ep_square: Option<Square>,
    draw_plies: usize,
    move_num: usize,
}

impl PositionBuilder {
    /// Creates a new, empty `PositionBuilder`
    pub fn new() -> Self {
        PositionBuilder {
            board: [ None; Square::COUNT ],
            turn: White,
            castle_king_side: [ false; Color::COUNT ],
            castle_queen_side: [ false; Color::COUNT ],
            ep_square: None,
            draw_plies: 0,
            move_num: 1,
        }
    }

    /// Sets the piece at `square`
    pub fn piece(&mut self, color: Color, piece: Piece, square: Square) -> &Self {
        self.board[square as usize] = Some((color, piece));
        self
    }

    /// Clears the piece at `square`
    pub fn clear(&mut self, square: Square) -> &Self {
        self.board[square as usize] = None;
        self
    }

    /// Sets the turn to `color`
    pub fn turn(&mut self, color: Color) -> &Self {
        self.turn = color;
        self
    }

    /// Sets king side castling rights for `color`
    pub fn can_castle_king_side(&mut self, color: Color, available: bool) -> &Self {
        self.castle_king_side[color as usize] = available;
        self
    }

    /// Sets queen side castling rights for `color`
    pub fn can_castle_queen_side(&mut self, color: Color, available: bool) -> &Self {
        self.castle_queen_side[color as usize] = available;
        self
    }

    /// Sets or clears the en-passant square
    pub fn en_passant_square(&mut self, square: Option<Square>) -> &Self {
        self.ep_square = square;
        self
    }

    /// Sets the number of plies that count toward the 50-move rule. A ply is a move by one player,
    /// so two plies would be one move by each player.
    pub fn draw_plies(&mut self, plies: usize) -> &Self {
        self.draw_plies = plies;
        self
    }

    /// Sets the move number
    pub fn move_number(&mut self, plies: usize) -> &Self {
        self.move_num = plies;
        self
    }

    /// Validates legality and returns a `Position`
    pub fn validate(&self) -> Result<Position> {
        use Error::*;

        let mut pos = Position::empty_board();

        for (i, piece) in self.board.iter().enumerate() {
            let sq = i.try_into().expect("INFALLIBLE");

            if let Some((color, piece)) = piece {
                // set the `sq` as occupied
                pos.occ_squares.insert(sq);
                pos.occ_by_color[*color as usize].insert(sq);
                pos.occ_by_piece[*color as usize][*piece as usize].insert(sq);
            }
        }

        pos.turn = self.turn;
        for c in 0..Color::COUNT {
            if self.castle_king_side[c] {
                pos.castling_rights[c] |= CASTLE_KING_SIDE;
            }
            if self.castle_queen_side[c] {
                pos.castling_rights[c] |= CASTLE_QUEEN_SIDE;
            }
        }
        pos.ep_square = self.ep_square;
        pos.draw_plies = self.draw_plies;
        pos.move_num = self.move_num;

        // validate position legality
        for c in &[White, Black] {
            // Step 1: verify exactly one king per side
            if pos.occupied_by_piece(*c, King).len() != 1 {
                return Err(InvalidKingCount);
            }
            // Step 2: no pawns on ranks 1 and 8
            if pos.occupied_by_piece(*c, Pawn)
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
                return Err(EnPassantSquareOccupied);
            }
            let forward = if pos.turn == White { 1 } else { -1 };
            if !pos.occupied_by_piece(!pos.turn, Pawn).shift_y(forward).contains(ep_square) {
                return Err(MissingEnPassantPawn);
            }
        }
        // Step 5: if castling rights exist, king and rook must be in the correct squares
        for c in &[White, Black] {
            if pos.has_castling_rights(*c) {
                let r = if *c == White { Rank::R1 } else { Rank::R8 };

                if !pos.occupied_by_piece(*c, King).contains(Square::from_coord(File::E, r)) {
                    return Err(InvalidCastlingFlags);
                }
                if pos.has_queen_side_castling_rights(*c)
                    && !pos.occupied_by_piece(*c, Rook).contains(Square::from_coord(File::A, r)) {
                    return Err(InvalidCastlingFlags);
                }
                if pos.has_king_side_castling_rights(*c)
                    && !pos.occupied_by_piece(*c, Rook).contains(Square::from_coord(File::H, r)) {
                    return Err(InvalidCastlingFlags);
                }
            }
        }

        pos.in_check = pos.square_attacked_by(pos.king_location(pos.turn), !pos.turn);
        pos.calc_zobrist();

        Ok(pos)
    }
}

impl Default for PositionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
