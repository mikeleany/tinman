//! Defines the error types needed by the chess module
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fmt;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Error type used by methods in the `Chess` module
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// Cannot parse string
    ParseError,
    /// Failed to convert an integer to an another type
    TryFromIntError,
    /// Ambiguous move
    AmbiguousMove,
    /// Illegal move
    IllegalMove,
    /// Player can capture opponent's king
    KingCapturable,
    /// Castling through check
    CastlingThroughCheck,
    /// Missing king or multiple kings of the same color
    InvalidKingCount,
    /// Pawn on first or last rank
    InvalidPawnRank,
    /// Castling flags aren't valid for this position
    InvalidCastlingFlags,
    /// En-passant square without capturable pawn
    MissingEnPassantPawn,
    /// En-passant square is occupied
    EnPassantSquareOccupied,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            ParseError => "cannot parse string",
            TryFromIntError => "integer out of range",
            AmbiguousMove => "ambiguous move",
            IllegalMove => "illegal move",
            CastlingThroughCheck => "attempt to castle through check",
            KingCapturable => "king is under attack on opponent's move",
            InvalidKingCount => "missing king or multiple kings of the same color",
            InvalidPawnRank => "pawn on first or last rank",
            InvalidCastlingFlags => "castling flags aren't valid for this position",
            MissingEnPassantPawn => "en-passant square without capturable pawn",
            EnPassantSquareOccupied => "en-passant square is occupied",
        }.fmt(f)
    }
}

impl std::error::Error for Error { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Result type used by methods in the `Chess` module
pub type Result<T> = std::result::Result<T, Error>;
