//! Function to evaluate a position.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::ops;
use crate::chess::{Color, Piece, Square, Position};

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Score
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(i16);

impl Score {
    /// Returns the greatest possible score
    pub fn infinity() -> Self {
        Score(10_000)
    }
    /// Returns the score for a draw
    pub fn draw() -> Self {
        Score(0)
    }
    /// Returns the score for checkmating in `n` plies
    pub fn mates_in(n: usize) -> Self {
        Score::infinity() - n as i16
    }
    /// Returns the score for being checkmated in `n` plies
    pub fn mated_in(n: usize) -> Self {
        -Score::infinity() + n as i16
    }
}

impl ops::Neg for Score {
    type Output = Score;

    fn neg(self) -> Self {
        Score(-self.0)
    }
}

impl ops::Add<i16> for Score {
    type Output = Score;

    fn add(self, rhs: i16) -> Self {
        Score(self.0 + rhs)
    }
}

impl ops::Sub<i16> for Score {
    type Output = Score;

    fn sub(self, rhs: i16) -> Self {
        Score(self.0 - rhs)
    }
}

impl From<i16> for Score {
    fn from(val: i16) -> Self {
        Score(val)
    }
}

impl From<Score> for i16 {
    fn from(val: Score) -> Self {
        val.0
    }
}

const PIECE_VAL: [i16; Piece::COUNT] = [ 100, 320, 330, 500, 1000, 0 ];

const PIECE_SQUARE_VAL: [[i16; Square::COUNT]; Piece::COUNT] = [
    [ // Pawn
      //  1    2    3    4    5    6    7    8
          0,   5,   4,  -5,   5,  10,  70,   0, // a
          0,  10,  -5,  -2,   7,  15,  70,   0, // b
          0,  10,  -5,   2,  10,  20,  70,   0, // c
          0, -25,   5,  15,  20,  30,  70,   0, // d
          0, -30,   4,  16,  20,  30,  70,   0, // e
          0,  10, -10,   0,  10,  20,  70,   0, // f
          0,  10,  -5,  -2,   7,  15,  70,   0, // g
          0,   5,   4,  -5,   5,  10,  70,   0, // h
    ],
    [ // Knight
      //  1    2    3    4    5    6    7    8
        -40, -30, -20, -20, -20, -20, -30, -40, // a
        -30, -10,   7,   5,   5,   7, -10, -30, // b
        -20,   0,  10,  15,  15,  12,   0, -20, // c
        -20,   5,  12,  20,  25,  15,   0, -20, // d
        -20,   5,  12,  20,  25,  15,   0, -20, // e
        -20,   0,  10,  15,  15,  12,   0, -20, // f
        -30, -10,   7,   5,   5,   7, -10, -30, // g
        -40, -30, -20, -20, -20, -20, -30, -40, // h
    ],  
    [ // Bishop
      //  1    2    3    4    5    6    7    8
        -20,  -7, -10, -10, -10, -10, -10, -20, // a
        -10,   5,  13,   5,   5,   0,   0, -10, // b
        -50,   0,  10,  13,   7,   5,   0, -10, // c
        -10,   0,   5,  10,  13,   7,   2, -10, // d
        -10,   0,   5,  10,  10,  10,   2, -10, // e
        -50,   0,  10,  10,   7,   5,   2, -10, // f
        -10,  15,  10,   5,   5,   0,   0, -10, // g
        -20, -10, -10, -10, -10, -10,  -7, -20, // h
    ],  
    [ // Rook
      //  1    2    3    4    5    6    7    8
        -20, -10,  10,  10,  10,  10,  20,  10, // a
        -10,   5,   5,   5,   5,   5,  30,  10, // b
         20,  10,   0,   0,   0,   0,  40,  20, // c
         30,  10,   0,   0,   0,   0,  50,  40, // d
         30,  10,   0,   0,   0,   0,  50,  40, // e
         20,  10,   0,   0,   0,   0,  40,  20, // f
        -20,   5,   5,   5,   5,   5,  30,  10, // g
        -30, -10,  10,  10,  10,  10,  20,  10, // h
    ],  
    [ 0; Square::COUNT ], // Queen
    [ 0; Square::COUNT ], // King
];

const MID_KING_TABLE: [i16; Square::COUNT] =  [
    //  1    2    3    4    5    6    7    8
     20,  10, -10, -30, -40, -50, -60, -70, // a
     30,  10, -20, -30, -40, -50, -60, -70, // b
     10,   0, -20, -30, -40, -50, -60, -70, // c
      0, -10, -20, -30, -40, -50, -60, -70, // d
      0, -10, -20, -30, -40, -50, -60, -70, // e
     10,   0, -20, -30, -40, -50, -60, -70, // f
     40,  10, -20, -30, -40, -50, -60, -70, // g
     20,  10, -10, -30, -40, -50, -60, -70, // h
];  

const END_KING_TABLE: [i16; Square::COUNT] =  [
    //  1    2    3    4    5    6    7    8
    -50, -40, -30, -20, -20, -30, -40, -50, // a
    -40, -30, -20, -10, -10, -20, -30, -40, // b
    -30, -20,  20,  30,  30,  20, -20, -30, // c
    -20, -10,  30,  50,  50,  30, -10, -20, // d
    -20, -10,  30,  50,  50,  30, -10, -20, // e
    -30, -20,  20,  30,  30,  20, -20, -30, // f
    -40, -30, -20, -10, -10, -20, -30, -40, // g
    -50, -40, -30, -20, -20, -30, -40, -50, // h
];  


/// Returns the value of a piece.
pub fn piece_val(piece: Piece) -> i16 {
    PIECE_VAL[piece as usize]
}

/// Returns the estimated static score for the current search position.
pub fn evaluate(pos: &Position) -> Score {
    use Color::*;
    use Piece::*;

    let mut val = [0; Color::COUNT];
    let mut total_piece_val = 0;

    let mut knights = [0; Color::COUNT];
    let mut bishops = [0; Color::COUNT];
    let mut good_pieces = [false; Color::COUNT];

    for color in [White, Black].iter().copied() {
        for piece in [Pawn, Knight, Bishop, Rook, Queen].iter().copied() {
            let mut count = 0;
            for sq in pos.occupied_by_piece(color, piece) {
                count += 1;
                let sq = if color == White { sq as usize } else { sq as usize ^ 0o07 };
                val[color as usize] += PIECE_VAL[piece as usize]
                    + PIECE_SQUARE_VAL[piece as usize][sq as usize];
            }
            total_piece_val += count * PIECE_VAL[piece as usize];

            if count > 0 {
                match piece {
                    Knight => knights[color as usize] = count,
                    Bishop => bishops[color as usize] = count,
                    _ => good_pieces[color as usize] = true,
                }
            }
        }
    }

    for color in [White, Black].iter().copied() {
        let sq = pos.occupied_by_piece(color, King).peek().expect("INFALLIBLE");
        let sq = if color == White { sq as usize } else { sq as usize ^ 0o07 };

        if total_piece_val > 3*PIECE_VAL[Queen as usize] {
            val[color as usize] += MID_KING_TABLE[sq as usize];
        } else if total_piece_val > 2*PIECE_VAL[Queen as usize] {
            val[color as usize] += (MID_KING_TABLE[sq as usize] + END_KING_TABLE[sq as usize])/2;
        } else {
            val[color as usize] += END_KING_TABLE[sq as usize];
        }
    }

    let val = val[pos.turn() as usize] - val[!pos.turn() as usize];

    let strong_side = if val > 0 { pos.turn() } else { !pos.turn() };
    let weak_side = !strong_side as usize;
    let strong_side = strong_side as usize;

    if good_pieces[strong_side]
    || bishops[strong_side] + knights[strong_side] > 2
    || (bishops[strong_side] == 2 && bishops[weak_side] == 0) {
        return val.into();
    } else if good_pieces[weak_side] || bishops[weak_side] > 0 || knights[weak_side] > 0 {
        return (val/25).into();
    } else {
        return 0.into();
    }
}

#[cfg(test)]
mod eval_test {
    use std::str::FromStr;
    use crate::chess::Position;
    use super::{Score, evaluate};

    #[test]
    fn eval() {
        assert_eq!(
            evaluate(&Position::from_str("k7/8/8/8/8/8/8/K7 w - - 0 1").unwrap()),
            Score::from(0)
        );
        assert_eq!(
            evaluate(&Position::from_str("kq6/8/8/8/8/8/8/K7 w - - 0 1").unwrap()),
            Score::from(-1000)
        );
        assert_eq!(
            evaluate(&Position::from_str("k7/8/8/8/8/8/8/KQ6 w - - 0 1").unwrap()),
            Score::from(1000)
        );
        assert_eq!(
            evaluate(&Position::from_str("k7/8/8/8/8/8/8/KQ6 b - - 0 1").unwrap()),
            Score::from(-1000)
        );
        assert_eq!(
            evaluate(&Position::from_str("k7/3p4/8/8/8/8/8/K7 b - - 0 1").unwrap()),
            evaluate(&Position::from_str("k7/8/8/8/8/8/3P4/K7 w - - 0 1").unwrap()),
        );
        assert_eq!(
            evaluate(&Position::from_str("k7/8/8/8/8/8/3P4/K7 w - - 0 1").unwrap()),
            -evaluate(&Position::from_str("k7/8/8/8/8/8/3P4/K7 b - - 0 1").unwrap()),
        );
    }
}
