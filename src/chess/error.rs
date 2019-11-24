//! Defines the error types needed by the chess module
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::error::Error;
use std::fmt;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a color
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "cannot parse color".fmt(f)
    }
}

impl Error for ParseColorError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a chess piece
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParsePieceError;

impl fmt::Display for ParsePieceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "cannot parse chess piece".fmt(f)
    }
}

impl Error for ParsePieceError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParseFileError;

impl fmt::Display for ParseFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "cannot parse file".fmt(f)
    }
}

impl Error for ParseFileError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a rank
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParseRankError;

impl fmt::Display for ParseRankError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "cannot parse rank".fmt(f)
    }
}

impl Error for ParseRankError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a square
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParseSquareError;

impl fmt::Display for ParseSquareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "cannot parse square".fmt(f)
    }
}

impl Error for ParseSquareError { }

impl From<ParseFileError> for ParseSquareError {
    fn from(_: ParseFileError) -> Self {
        ParseSquareError
    }
}

impl From<ParseRankError> for ParseSquareError {
    fn from(_: ParseRankError) -> Self {
        ParseSquareError
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in parsing a move string
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseMoveError {
    /// Cannot parse move
    ParseError,
    /// Ambiguous move, can't determine which piece to move
    AmbiguousMove,
    /// Illegal move
    IllegalMove,
}

impl fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseMoveError::ParseError => "cannot parse move",
            ParseMoveError::AmbiguousMove => "ambiguous move",
            ParseMoveError::IllegalMove => "illegal move",
        }.fmt(f)
    }
}

impl Error for ParseMoveError { }

impl From<ParseFileError> for ParseMoveError {
    fn from(_: ParseFileError) -> Self {
        ParseMoveError::ParseError
    }
}

impl From<ParseRankError> for ParseMoveError {
    fn from(_: ParseRankError) -> Self {
        ParseMoveError::ParseError
    }
}

impl From<TryFromIntError> for ParseMoveError {
    fn from(_: TryFromIntError) -> Self {
        ParseMoveError::ParseError
    }
}

impl From<ValidateMoveError> for ParseMoveError {
    fn from(_: ValidateMoveError) -> Self {
        ParseMoveError::IllegalMove
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error that can be returned when parsing a position from a FEN string
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseFenError {
    /// FEN string is empty
    Empty,
    /// Cannot parse board data
    ParseBoard,
    /// Cannot parse turn color
    ParseTurn,
    /// Cannot parse castling flags
    ParseCastling,
    /// Cannot parse en-passant square
    ParseEnPassant,
    /// Cannot parse half-move clock
    ParseHalfMoveClock,
    /// Cannot parse move number
    ParseMoveNumber,
    /// Missing king or multiple kings of the same color
    KingCount,
    /// Player can capture opponents king
    KingCapturable,
    /// Pawn on first or last rank
    InvalidPawnRank,
    /// Castling flags aren't valid for this position
    InvalidCastling,
    /// En-passant square without capturable pawn
    EnPassantPawn,
}

impl fmt::Display for ParseFenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParseFenError::Empty => "fen string is empty",
            ParseFenError::ParseBoard => "cannot parse board data",
            ParseFenError::ParseTurn => "cannot parse color",
            ParseFenError::ParseCastling => "cannot parse castling flags",
            ParseFenError::ParseEnPassant => "cannot parse en-passant square",
            ParseFenError::ParseHalfMoveClock => "cannot parse half-move clock",
            ParseFenError::ParseMoveNumber => "cannot parse move number",
            ParseFenError::KingCount => "missing king or multiple kings of the same color",
            ParseFenError::KingCapturable => "player can capture opponents king",
            ParseFenError::InvalidPawnRank => "pawn on first or last rank",
            ParseFenError::InvalidCastling => "castling flags aren't valid for this position",
            ParseFenError::EnPassantPawn => "en-passant square without capturable pawn",
        };

        s.fmt(f)
    }
}

impl Error for ParseFenError { }

impl From<ParsePieceError> for ParseFenError {
    fn from(_: ParsePieceError) -> Self {
        ParseFenError::ParseBoard
    }
}

impl From<ParseColorError> for ParseFenError {
    fn from(_: ParseColorError) -> Self {
        ParseFenError::ParseTurn
    }
}

impl From<ParseSquareError> for ParseFenError {
    fn from(_: ParseSquareError) -> Self {
        ParseFenError::ParseEnPassant
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in converting an integer to an another type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromIntError;

impl fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "integer out of range".fmt(f)
    }
}

impl Error for TryFromIntError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in validating a move
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValidateMoveError;

impl fmt::Display for ValidateMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "illegal move".fmt(f)
    }
}

impl Error for ValidateMoveError { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An error in making a move
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MakeMoveError {
    /// Castling through check
    CastlingThroughCheck,
    /// Mover's king is under attack
    SelfCheck,
}

impl fmt::Display for MakeMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MakeMoveError::CastlingThroughCheck => "cannot castle through check",
            MakeMoveError::SelfCheck => "mover is in check",
        }.fmt(f)
    }
}

impl Error for MakeMoveError { }
