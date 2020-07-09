//! The `chess` crate implements the
//! [FIDE Laws of Chess](https://www.fide.com/FIDE/handbook/LawsOfChess.pdf).
//!
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! The primary purpose behind this crate is to generate and validate moves from any given position.
//! As such, the [`Move`][Move] and the [`Position`][Position] are the core data structures of the
//! `chess` crate. It also contains the [`game`][game_mod] module for combining these together into
//! [`MoveSequence`][MoveSequence]s and [`Game`][Game]s.
//!
//! # Positions
//! The [`Position`][Position] structure is used to represent the arrangement of pieces on the
//! board, along with meta-data, such as castling rights. The board representation used is the
//! [`Bitboard`][Bitboard], and many of [`Position`][Position]'s routines return
//! [`Bitboard`][Bitboard]s. A [`Position`][Position] can be built using a
//! [`PositionBuilder`][PositionBuilder] or with [`Position`][Position]'s constructor methods
//! [`new`][position_new] and [`from_fen_str`][from_fen_str].
//!
//! # Moves
//! Every [`Move`][Move] is composed of a moved [`Piece`][Piece] an origin [`Square`][Square] and a
//! destination [`Square`][Square]. Some moves will also have a captured [`Piece`][Piece] and/or
//! a [`Promotion`][Promotion] piece. To keep track of where the move is valid, it also need to keep
//! a reference to the [`Position`][Position] where it is valid. [`Move`][Move] structures can be
//! built using [`MoveBuilder`][MoveBuilder].
//!
//! Since carrying a reference around, means that a [`Move`][Move] cannot outlive the
//! [`Position`][Position] where it's valid, a more versatile alternative, [`MoveRc`][MoveRc], is
//! also provided. As its name implies, rather than having a reference to a
//! [`Position`][Position], [`MoveRc`][MoveRc] uses an [`Rc<Position>`][Rc] instead. Both
//! [`Move`][Move] and [`MoveRc`][MoveRc] implement the [`ValidMove`][ValidMove] trait, which
//! provides all the methods necessary to work with moves.
//!
//! # Move Generation
//! As mentioned previously, the primary purpose behind the `chess` crate is to generate and
//! validate moves. [`MoveBuilder`][MoveBuilder] can be used for move validation, and
//! [`Position`][Position] provides some routines for the purpose of move generation. They are
//! [`moves`][moves], which returns an iterator over all valid moves, and
//! [`promotions_and_captures`][prom_and_capt], which, as its name suggests, returns an iterator
//! over just promotions and captures.
//!
//! A distinction should be noted here between valid and legal moves. So far the term legal has not
//! been used to describe moves. That is because verifying full legality is more expensive than
//! verifying psuedo-legality, or validity. Within Tinman's code and documentation, if a move is
//! referred to here as valid, that means it is legal if and only if the king is not left in
//! check, and, for castling moves, if it does not pass through an attacked square. To verify full
//! legality of a [`ValidMove`][ValidMove], the move must be made, using the [`make`][make] method.
//!
//! [Piece]: enum.Piece.html
//! [Square]: enum.Square.html
//! [Promotion]: enum.Promotion.html
//! [Move]: struct.Move.html
//! [MoveRc]: struct.MoveRc.html
//! [ValidMove]: trait.ValidMove.html
//! [make]: trait.ValidMove.html#method.make
//! [MoveBuilder]: struct.MoveBuilder.html
//! [Position]: struct.Position.html
//! [position_new]: struct.Position.html#method.new
//! [from_fen_str]: struct.Position.html#method.from_fen_str
//! [moves]: struct.Position.html#method.moves
//! [prom_and_capt]: struct.Position.html#method.promotions_and_captures
//! [PositionBuilder]: struct.PositionBuilder.html
//! [Bitboard]: bitboard/struct.Bitboard.html
//! [game_mod]: game/index.html
//! [MoveSequence]: game/struct.MoveSequence.html
//! [Game]: game/struct.Game.html
//! [Rc]: https://doc.rust-lang.org/std/rc/struct.Rc.html
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

use std::ops;
use std::fmt;
use std::mem;
use std::str::FromStr;
use std::convert::TryFrom;
pub use error::*;


////////////////////////////////////////////////////////////////////////////////////////////////////
/// Which side a piece or player is on, based on the color of the pieces for that side.
///
/// `Color` implements [`Not`][Not], so that we can do the following:
///
/// ```rust
/// use chess::Color;
/// 
/// let mut turn = Color::White;
/// turn = !turn;
/// assert_eq!(turn, Color::Black);
///
/// turn = !turn;
/// assert_eq!(turn, Color::White);
/// ```
///
/// `Color` also implements [`Display`][Display] and [`FromStr`][FromStr].
///
/// ```rust
/// use chess::Color;
/// use std::str::FromStr;
///
/// assert_eq!(Color::from_str("w"), Ok(Color::White));
/// assert_eq!(Color::from_str("b"), Ok(Color::Black));
///
/// assert_eq!(format!("{}", Color::White), "w");
/// assert_eq!(format!("{}", Color::Black), "b");
/// ```
///
/// [Not]: https://doc.rust-lang.org/std/ops/trait.Not.html
/// [Display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [FromStr]: https://doc.rust-lang.org/std/str/trait.FromStr.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// The number of colors
    pub const COUNT: usize = 2;
}

impl ops::Not for Color {
    type Output = Color;

    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => 'w'.fmt(f),
            Color::Black => 'b'.fmt(f),
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _   => Err(Error::ParseError),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::White
    }
}

impl TryFrom<usize> for Color {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        if value < Self::COUNT {
            unsafe { Ok(mem::transmute::<u8, Color>(value as u8)) }
        } else {
            Err(Error::TryFromIntError)
        }
    }
}

impl From<Color> for usize {
    fn from(value: Color) -> Self {
        value as Self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// The type of a chess piece.
///
/// The [`Display`][Display] trait produces uppercase letters.
///
/// ```
/// use chess::Piece;
/// assert_eq!(format!("{}", Piece::Pawn), "P");
/// assert_eq!(format!("{}", Piece::Knight), "N");
/// assert_eq!(format!("{}", Piece::Bishop), "B");
/// assert_eq!(format!("{}", Piece::Rook), "R");
/// assert_eq!(format!("{}", Piece::Queen), "Q");
/// assert_eq!(format!("{}", Piece::King), "K");
/// ```
///
/// However, [`FromStr`][FromStr] can parse uppercase or lowercase.
///
/// ```rust
/// use chess::Piece;
/// use std::str::FromStr;
///
/// assert_eq!(Piece::from_str("P"), Ok(Piece::Pawn));
/// assert_eq!(Piece::from_str("N"), Ok(Piece::Knight));
/// assert_eq!(Piece::from_str("B"), Ok(Piece::Bishop));
/// assert_eq!(Piece::from_str("R"), Ok(Piece::Rook));
/// assert_eq!(Piece::from_str("Q"), Ok(Piece::Queen));
/// assert_eq!(Piece::from_str("K"), Ok(Piece::King));
///
/// assert_eq!(Piece::from_str("p"), Ok(Piece::Pawn));
/// assert_eq!(Piece::from_str("n"), Ok(Piece::Knight));
/// assert_eq!(Piece::from_str("b"), Ok(Piece::Bishop));
/// assert_eq!(Piece::from_str("r"), Ok(Piece::Rook));
/// assert_eq!(Piece::from_str("q"), Ok(Piece::Queen));
/// assert_eq!(Piece::from_str("k"), Ok(Piece::King));
/// ```
///
/// [Display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [FromStr]: https://doc.rust-lang.org/std/str/trait.FromStr.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    /// The number of piece types
    pub const COUNT: usize = Piece::King as usize + 1;
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Pawn => "P",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Rook => "R",
            Piece::Queen => "Q",
            Piece::King => "K",
        }.fmt(f)
    }
}

impl FromStr for Piece {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "P"|"p" => Ok(Piece::Pawn),
            "N"|"n" => Ok(Piece::Knight),
            "B"|"b" => Ok(Piece::Bishop),
            "R"|"r" => Ok(Piece::Rook),
            "Q"|"q" => Ok(Piece::Queen),
            "K"|"k" => Ok(Piece::King),
            _       => Err(Error::ParseError),
        }
    }
}

impl Default for Piece {
    fn default() -> Self {
        Piece::Pawn
    }
}

impl TryFrom<usize> for Piece {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        if value < Self::COUNT {
            unsafe { Ok(mem::transmute::<u8, Piece>(value as u8)) }
        } else {
            Err(Error::TryFromIntError)
        }
    }
}

impl From<Piece> for usize {
    fn from(value: Piece) -> Self {
        value as Self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Vertical column of the board, labeled from left to right from `White`'s perspective as
/// `A` through `H`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum File {
    // discriminants are spelled out so nothing can go wrong when we use transmute later
    A = 0, B = 1, C = 2, D = 3, E = 4, F = 5, G = 6, H = 7,
}

impl File {
    /// The number of files
    pub const COUNT: usize = File::H as usize + 1;
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            File::A => "a",
            File::B => "b",
            File::C => "c",
            File::D => "d",
            File::E => "e",
            File::F => "f",
            File::G => "g",
            File::H => "h",
        }.fmt(f)
    }
}

impl FromStr for File {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "a" => Ok(File::A),
            "b" => Ok(File::B),
            "c" => Ok(File::C),
            "d" => Ok(File::D),
            "e" => Ok(File::E),
            "f" => Ok(File::F),
            "g" => Ok(File::G),
            "h" => Ok(File::H),
            _       => Err(Error::ParseError),
        }
    }
}

impl Default for File {
    fn default() -> Self {
        File::A
    }
}

impl TryFrom<usize> for File {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        if value < Self::COUNT {
            unsafe { Ok(mem::transmute::<u8, File>(value as u8)) }
        } else {
            Err(Error::TryFromIntError)
        }
    }
}

impl From<File> for usize {
    fn from(value: File) -> Self {
        value as Self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Horizontal row of the board, labeled from nearest to farthest from `White`'s perspective
/// as `R1` through `R8`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Rank {
    // discriminants are spelled out so nothing can go wrong when we use transmute later
    R1 = 0, R2 = 1, R3 = 2, R4 = 3, R5 = 4, R6 = 5, R7 = 6, R8 = 7,
}

impl Rank {
    /// The number of ranks
    pub const COUNT: usize = Rank::R8 as usize + 1;
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rank::R1 => "1",
            Rank::R2 => "2",
            Rank::R3 => "3",
            Rank::R4 => "4",
            Rank::R5 => "5",
            Rank::R6 => "6",
            Rank::R7 => "7",
            Rank::R8 => "8",
        }.fmt(f)
    }
}

impl FromStr for Rank {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1" => Ok(Rank::R1),
            "2" => Ok(Rank::R2),
            "3" => Ok(Rank::R3),
            "4" => Ok(Rank::R4),
            "5" => Ok(Rank::R5),
            "6" => Ok(Rank::R6),
            "7" => Ok(Rank::R7),
            "8" => Ok(Rank::R8),
            _       => Err(Error::ParseError),
        }
    }
}

impl Default for Rank {
    fn default() -> Self {
        Rank::R1
    }
}

impl TryFrom<usize> for Rank {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        if value < Self::COUNT {
            unsafe { Ok(mem::transmute::<u8, Rank>(value as u8)) }
        } else {
            Err(Error::TryFromIntError)
        }
    }
}

impl From<Rank> for usize {
    fn from(value: Rank) -> Self {
        value as Self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A specific square on the board, labeled using the `File` and `Rank` as coordinates.
///
/// In addition to using variants, a `Square` can be constructed from its `File` and `Rank`.
///
/// ```rust
/// use chess::{Square, File, Rank};
///
/// assert_eq!(Square::from_coord(File::A, Rank::R1), Square::A1);
/// ```
///
/// It can also be decomposed as follows:
///
/// ```rust
/// use chess::{Square, File, Rank};
///
/// let square = Square::A1;
/// assert_eq!(square.file(), File::A);
/// assert_eq!(square.rank(), Rank::R1);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Square {
    // discriminants are spelled out so nothing can go wrong when we use transmute later
    A1 = 0o00, A2 = 0o01, A3 = 0o02, A4 = 0o03, A5 = 0o04, A6 = 0o05, A7 = 0o06, A8 = 0o07,
    B1 = 0o10, B2 = 0o11, B3 = 0o12, B4 = 0o13, B5 = 0o14, B6 = 0o15, B7 = 0o16, B8 = 0o17,
    C1 = 0o20, C2 = 0o21, C3 = 0o22, C4 = 0o23, C5 = 0o24, C6 = 0o25, C7 = 0o26, C8 = 0o27,
    D1 = 0o30, D2 = 0o31, D3 = 0o32, D4 = 0o33, D5 = 0o34, D6 = 0o35, D7 = 0o36, D8 = 0o37,
    E1 = 0o40, E2 = 0o41, E3 = 0o42, E4 = 0o43, E5 = 0o44, E6 = 0o45, E7 = 0o46, E8 = 0o47,
    F1 = 0o50, F2 = 0o51, F3 = 0o52, F4 = 0o53, F5 = 0o54, F6 = 0o55, F7 = 0o56, F8 = 0o57,
    G1 = 0o60, G2 = 0o61, G3 = 0o62, G4 = 0o63, G5 = 0o64, G6 = 0o65, G7 = 0o66, G8 = 0o67,
    H1 = 0o70, H2 = 0o71, H3 = 0o72, H4 = 0o73, H5 = 0o74, H6 = 0o75, H7 = 0o76, H8 = 0o77,
}

impl Square {
    /// The number of squares
    pub const COUNT: usize = Square::H8 as usize + 1;

    /// Returns a square from its file and rank
    pub fn from_coord(file: File, rank: Rank) -> Square {
        Square::try_from(((file as usize) << 3) + rank as usize).expect("INFALLIBLE")
    }

    /// Returns the square's file
    pub fn file(self) -> File {
        File::try_from((self as usize) >> 3).expect("INFALLIBLE")
    }

    /// Returns the square's rank
    pub fn rank(self) -> Rank {
        Rank::try_from((self as usize) & 7).expect("INFALLIBLE")
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.file().to_string() + &self.rank().to_string()).fmt(f)
    }
}

impl FromStr for Square {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let c: Vec<_> = s.chars().collect();
        if c.len() == 2 {
            Ok(Square::from_coord(c[0].to_string().parse()?, c[1].to_string().parse()?))
        } else {
            Err(Error::ParseError)
        }
    }
}

impl Default for Square {
    fn default() -> Self {
        Square::A1
    }
}

impl TryFrom<usize> for Square {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        if value < Self::COUNT {
            unsafe { Ok(mem::transmute::<u8, Square>(value as u8)) }
        } else {
            Err(Error::TryFromIntError)
        }
    }
}

impl From<Square> for usize {
    fn from(value: Square) -> Self {
        value as Self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod bitboard;
mod moves;
pub use moves::{ValidMove, Move, MoveRc, MoveType, Promotion, MoveBuilder};
mod position;
pub use position::Position;
pub use position::builder::PositionBuilder;
pub use position::zobrist::Zobrist;
pub use position::move_iter::{Moves, PromotionsAndCaptures};
pub mod game;

pub mod variations;

mod error;

#[cfg(test)]
mod color_tests {
    use std::convert::TryFrom;
    use super::Color;

    #[test]
    fn default_is_white() {
        assert_eq!(Color::White, Default::default());
    }

    #[test]
    fn to_usize_is_correct() {
        assert_eq!(usize::from(Color::White), 0);
        assert_eq!(usize::from(Color::Black), 1);
    }

    #[test]
    fn from_usize_is_correct() {
        assert_eq!(Color::try_from(0).unwrap(), Color::White);
        assert_eq!(Color::try_from(1).unwrap(), Color::Black);
        assert!(Color::try_from(2).is_err());
    }
}

#[cfg(test)]
mod piece_tests {
    use std::convert::TryFrom;
    use super::Piece;

    #[test]
    fn default_is_pawn() {
        assert_eq!(Piece::Pawn, Default::default());
    }

    #[test]
    fn to_usize_is_correct() {
        assert_eq!(usize::from(Piece::Pawn), 0);
        assert_eq!(usize::from(Piece::Knight), 1);
        assert_eq!(usize::from(Piece::Bishop), 2);
        assert_eq!(usize::from(Piece::Rook), 3);
        assert_eq!(usize::from(Piece::Queen), 4);
        assert_eq!(usize::from(Piece::King), 5);
    }

    #[test]
    fn from_usize_is_correct() {
        assert_eq!(Piece::try_from(0).unwrap(), Piece::Pawn);
        assert_eq!(Piece::try_from(1).unwrap(), Piece::Knight);
        assert_eq!(Piece::try_from(2).unwrap(), Piece::Bishop);
        assert_eq!(Piece::try_from(3).unwrap(), Piece::Rook);
        assert_eq!(Piece::try_from(4).unwrap(), Piece::Queen);
        assert_eq!(Piece::try_from(5).unwrap(), Piece::King);
        assert!(Piece::try_from(6).is_err());
    }
}

#[cfg(test)]
mod file_tests {
    use std::convert::TryFrom;
    use super::File;

    #[test]
    fn display_trait_works() {
        assert_eq!(format!("{}", File::A), "a");
        assert_eq!(format!("{}", File::B), "b");
        assert_eq!(format!("{}", File::C), "c");
        assert_eq!(format!("{}", File::D), "d");
        assert_eq!(format!("{}", File::E), "e");
        assert_eq!(format!("{}", File::F), "f");
        assert_eq!(format!("{}", File::G), "g");
        assert_eq!(format!("{}", File::H), "h");
    }

    #[test]
    fn fromstr_trait_works() {
        assert_eq!("a".parse::<File>().unwrap(), File::A);
        assert_eq!("b".parse::<File>().unwrap(), File::B);
        assert_eq!("c".parse::<File>().unwrap(), File::C);
        assert_eq!("d".parse::<File>().unwrap(), File::D);
        assert_eq!("e".parse::<File>().unwrap(), File::E);
        assert_eq!("f".parse::<File>().unwrap(), File::F);
        assert_eq!("g".parse::<File>().unwrap(), File::G);
        assert_eq!("h".parse::<File>().unwrap(), File::H);
        assert!("x".parse::<File>().is_err());
    }

    #[test]
    fn default_is_file_a() {
        assert_eq!(File::A, Default::default());
    }

    #[test]
    fn to_usize_is_correct() {
        assert_eq!(usize::from(File::A), 0);
        assert_eq!(usize::from(File::B), 1);
        assert_eq!(usize::from(File::C), 2);
        assert_eq!(usize::from(File::D), 3);
        assert_eq!(usize::from(File::E), 4);
        assert_eq!(usize::from(File::F), 5);
        assert_eq!(usize::from(File::G), 6);
        assert_eq!(usize::from(File::H), 7);
    }

    #[test]
    fn from_usize_is_correct() {
        assert_eq!(File::try_from(0).unwrap(), File::A);
        assert_eq!(File::try_from(1).unwrap(), File::B);
        assert_eq!(File::try_from(2).unwrap(), File::C);
        assert_eq!(File::try_from(3).unwrap(), File::D);
        assert_eq!(File::try_from(4).unwrap(), File::E);
        assert_eq!(File::try_from(5).unwrap(), File::F);
        assert_eq!(File::try_from(6).unwrap(), File::G);
        assert_eq!(File::try_from(7).unwrap(), File::H);
        assert!(File::try_from(8).is_err());
    }
}

#[cfg(test)]
mod rank_tests {
    use std::convert::TryFrom;
    use super::Rank;

    #[test]
    fn display_trait_works() {
        assert_eq!(format!("{}", Rank::R1), "1");
        assert_eq!(format!("{}", Rank::R2), "2");
        assert_eq!(format!("{}", Rank::R3), "3");
        assert_eq!(format!("{}", Rank::R4), "4");
        assert_eq!(format!("{}", Rank::R5), "5");
        assert_eq!(format!("{}", Rank::R6), "6");
        assert_eq!(format!("{}", Rank::R7), "7");
        assert_eq!(format!("{}", Rank::R8), "8");
    }

    #[test]
    fn fromstr_trait_works() {
        assert_eq!("1".parse::<Rank>().unwrap(), Rank::R1);
        assert_eq!("2".parse::<Rank>().unwrap(), Rank::R2);
        assert_eq!("3".parse::<Rank>().unwrap(), Rank::R3);
        assert_eq!("4".parse::<Rank>().unwrap(), Rank::R4);
        assert_eq!("5".parse::<Rank>().unwrap(), Rank::R5);
        assert_eq!("6".parse::<Rank>().unwrap(), Rank::R6);
        assert_eq!("7".parse::<Rank>().unwrap(), Rank::R7);
        assert_eq!("8".parse::<Rank>().unwrap(), Rank::R8);
        assert!("x".parse::<Rank>().is_err());
    }

    #[test]
    fn default_is_rank_1() {
        assert_eq!(Rank::R1, Default::default());
    }

    #[test]
    fn to_usize_is_correct() {
        // test usize conversions
        assert_eq!(usize::from(Rank::R1), 0);
        assert_eq!(usize::from(Rank::R2), 1);
        assert_eq!(usize::from(Rank::R3), 2);
        assert_eq!(usize::from(Rank::R4), 3);
        assert_eq!(usize::from(Rank::R5), 4);
        assert_eq!(usize::from(Rank::R6), 5);
        assert_eq!(usize::from(Rank::R7), 6);
        assert_eq!(usize::from(Rank::R8), 7);
    }

    #[test]
    fn from_usize_is_correct() {
        assert_eq!(Rank::try_from(0).unwrap(), Rank::R1);
        assert_eq!(Rank::try_from(1).unwrap(), Rank::R2);
        assert_eq!(Rank::try_from(2).unwrap(), Rank::R3);
        assert_eq!(Rank::try_from(3).unwrap(), Rank::R4);
        assert_eq!(Rank::try_from(4).unwrap(), Rank::R5);
        assert_eq!(Rank::try_from(5).unwrap(), Rank::R6);
        assert_eq!(Rank::try_from(6).unwrap(), Rank::R7);
        assert_eq!(Rank::try_from(7).unwrap(), Rank::R8);
        assert!(Rank::try_from(8).is_err());
    }
}

#[cfg(test)]
mod square_tests {
    use std::convert::TryFrom;
    use super::File;
    use super::Rank;
    use super::Square;

    #[test]
    fn from_coord_constructor_matches_variant_names() {
        assert_eq!(Square::from_coord(File::A, Rank::R1), Square::A1);
        assert_eq!(Square::from_coord(File::B, Rank::R1), Square::B1);
        assert_eq!(Square::from_coord(File::C, Rank::R1), Square::C1);
        assert_eq!(Square::from_coord(File::D, Rank::R1), Square::D1);
        assert_eq!(Square::from_coord(File::E, Rank::R1), Square::E1);
        assert_eq!(Square::from_coord(File::F, Rank::R1), Square::F1);
        assert_eq!(Square::from_coord(File::G, Rank::R1), Square::G1);
        assert_eq!(Square::from_coord(File::H, Rank::R1), Square::H1);
        assert_eq!(Square::from_coord(File::A, Rank::R2), Square::A2);
        assert_eq!(Square::from_coord(File::B, Rank::R2), Square::B2);
        assert_eq!(Square::from_coord(File::C, Rank::R2), Square::C2);
        assert_eq!(Square::from_coord(File::D, Rank::R2), Square::D2);
        assert_eq!(Square::from_coord(File::E, Rank::R2), Square::E2);
        assert_eq!(Square::from_coord(File::F, Rank::R2), Square::F2);
        assert_eq!(Square::from_coord(File::G, Rank::R2), Square::G2);
        assert_eq!(Square::from_coord(File::H, Rank::R2), Square::H2);
        assert_eq!(Square::from_coord(File::A, Rank::R3), Square::A3);
        assert_eq!(Square::from_coord(File::B, Rank::R3), Square::B3);
        assert_eq!(Square::from_coord(File::C, Rank::R3), Square::C3);
        assert_eq!(Square::from_coord(File::D, Rank::R3), Square::D3);
        assert_eq!(Square::from_coord(File::E, Rank::R3), Square::E3);
        assert_eq!(Square::from_coord(File::F, Rank::R3), Square::F3);
        assert_eq!(Square::from_coord(File::G, Rank::R3), Square::G3);
        assert_eq!(Square::from_coord(File::H, Rank::R3), Square::H3);
        assert_eq!(Square::from_coord(File::A, Rank::R4), Square::A4);
        assert_eq!(Square::from_coord(File::B, Rank::R4), Square::B4);
        assert_eq!(Square::from_coord(File::C, Rank::R4), Square::C4);
        assert_eq!(Square::from_coord(File::D, Rank::R4), Square::D4);
        assert_eq!(Square::from_coord(File::E, Rank::R4), Square::E4);
        assert_eq!(Square::from_coord(File::F, Rank::R4), Square::F4);
        assert_eq!(Square::from_coord(File::G, Rank::R4), Square::G4);
        assert_eq!(Square::from_coord(File::H, Rank::R4), Square::H4);
        assert_eq!(Square::from_coord(File::A, Rank::R5), Square::A5);
        assert_eq!(Square::from_coord(File::B, Rank::R5), Square::B5);
        assert_eq!(Square::from_coord(File::C, Rank::R5), Square::C5);
        assert_eq!(Square::from_coord(File::D, Rank::R5), Square::D5);
        assert_eq!(Square::from_coord(File::E, Rank::R5), Square::E5);
        assert_eq!(Square::from_coord(File::F, Rank::R5), Square::F5);
        assert_eq!(Square::from_coord(File::G, Rank::R5), Square::G5);
        assert_eq!(Square::from_coord(File::H, Rank::R5), Square::H5);
        assert_eq!(Square::from_coord(File::A, Rank::R6), Square::A6);
        assert_eq!(Square::from_coord(File::B, Rank::R6), Square::B6);
        assert_eq!(Square::from_coord(File::C, Rank::R6), Square::C6);
        assert_eq!(Square::from_coord(File::D, Rank::R6), Square::D6);
        assert_eq!(Square::from_coord(File::E, Rank::R6), Square::E6);
        assert_eq!(Square::from_coord(File::F, Rank::R6), Square::F6);
        assert_eq!(Square::from_coord(File::G, Rank::R6), Square::G6);
        assert_eq!(Square::from_coord(File::H, Rank::R6), Square::H6);
        assert_eq!(Square::from_coord(File::A, Rank::R7), Square::A7);
        assert_eq!(Square::from_coord(File::B, Rank::R7), Square::B7);
        assert_eq!(Square::from_coord(File::C, Rank::R7), Square::C7);
        assert_eq!(Square::from_coord(File::D, Rank::R7), Square::D7);
        assert_eq!(Square::from_coord(File::E, Rank::R7), Square::E7);
        assert_eq!(Square::from_coord(File::F, Rank::R7), Square::F7);
        assert_eq!(Square::from_coord(File::G, Rank::R7), Square::G7);
        assert_eq!(Square::from_coord(File::H, Rank::R7), Square::H7);
        assert_eq!(Square::from_coord(File::A, Rank::R8), Square::A8);
        assert_eq!(Square::from_coord(File::B, Rank::R8), Square::B8);
        assert_eq!(Square::from_coord(File::C, Rank::R8), Square::C8);
        assert_eq!(Square::from_coord(File::D, Rank::R8), Square::D8);
        assert_eq!(Square::from_coord(File::E, Rank::R8), Square::E8);
        assert_eq!(Square::from_coord(File::F, Rank::R8), Square::F8);
        assert_eq!(Square::from_coord(File::G, Rank::R8), Square::G8);
        assert_eq!(Square::from_coord(File::H, Rank::R8), Square::H8);
    }

    #[test]
    fn file_and_rank_methods_match_from_coord() {
        for f in vec![ File::A, File::B, File::C, File::D,
                    File::E, File::F, File::G, File::H ] {
            for r in vec![ Rank::R1, Rank::R2, Rank::R3, Rank::R4,
                        Rank::R5, Rank::R6, Rank::R7, Rank::R8 ] {
                let s = Square::from_coord(f, r);
                assert_eq!(f, s.file());
                assert_eq!(r, s.rank());
            }
        }
    }

    #[test]
    fn display_and_fromstr_traits_match_file_and_rank() {
        for f in vec![ File::A, File::B, File::C, File::D,
                    File::E, File::F, File::G, File::H ] {
            for r in vec![ Rank::R1, Rank::R2, Rank::R3, Rank::R4,
                        Rank::R5, Rank::R6, Rank::R7, Rank::R8 ] {
                let s = Square::from_coord(f, r);
                assert_eq!(format!("{}", s), format!("{}{}", f, r));
                assert_eq!(format!("{}", s).parse::<Square>().unwrap(), s);
            }
        }
    }

    #[test]
    fn fromstr_trait_produces_errors_when_it_should() {
        assert!("a".parse::<Square>().is_err());
        assert!("1".parse::<Square>().is_err());
        assert!("ax".parse::<Square>().is_err());
        assert!("x1".parse::<Square>().is_err());
        assert!("a1x".parse::<Square>().is_err());
    }

    #[test]
    fn default_is_a1() {
        // test Default trait
        assert_eq!(Square::A1, Default::default());
    }

    #[test]
    fn usize_conversions_are_consistent() {
        // test usize conversions
        for i in 0..Square::COUNT {
            let s = Square::try_from(i).unwrap();
            assert_eq!(s as usize, i);
            assert_eq!(usize::from(s), i);
        }
    }

    #[test]
    fn out_of_bound_usize_conversion_is_an_error() {
        assert!(Square::try_from(Square::COUNT).is_err());
    }
}

