//! Provides a representation of the pieces on the board
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! # Moves and Attacks
//! Bitboards are useful for quickly computing the moves or attacks available to a piece based on
//! its location on the board. In addition to the [`Bitboard`](struct.Bitboard.html) type, the
//! `bitboard` module also provides functions to compute moves and attacks for all pieces except
//! pawns. For these pieces, the word "attacks" is used here as these pieces can only move to the
//! squares where they attack.
//!
//! ## Direct attacks (Knights and Kings)
//! Knights and kings move directly to their destinations without passing through any other squares.
//! That makes computing these attacks a bit easier than with the sliding pieces. For example, the
//! squares attacked by a knight on h1 can be computed as follows:
//!
//! ```rust
//! use chess::Square;
//! use chess::bitboard::knight_attacks;
//!
//! let mut attacks = knight_attacks(Square::H1);
//! assert_eq!(attacks.pop(), Some(Square::F2));
//! assert_eq!(attacks.pop(), Some(Square::G3));
//! assert_eq!(attacks.pop(), None);
//! ```
//!
//! King attacks can be computed in the same way.
//!
//! ## Sliding Attacks (Bishops, Rooks and Queens)
//! Moves by sliding pieces can be blocked by pieces in the path. For this reason, the functions for
//! sliding attacks require an additional argument: a `Bitboard` of occupied squares. Here's an
//! example of rook attacks:
//!
//! ```rust
//! use chess::Square;
//! use chess::bitboard::{Bitboard, rook_attacks};
//!
//! let occ = Bitboard::from(Square::A2) | Square::C1.into();
//! let mut attacks = rook_attacks(Square::A1, occ);
//! assert_eq!(attacks.pop(), Some(Square::A2));
//! assert_eq!(attacks.pop(), Some(Square::B1));
//! assert_eq!(attacks.pop(), Some(Square::C1));
//! assert_eq!(attacks.pop(), None);
//! ```
//!
//! Bishop and queen attacks can be computed in the same way.
//!
//! ## Pawn Advancements and Attacks
//! For pawns, there's not a function like those used for other piece attacks. Instead the
//! advancements and attacks of multiple pawns can be computed simultaneously using the
//! [`Bitboard::shift_y`](struct.Bitboard.html#method.shift_y) and
//! [`Bityboard::shift_xy`](struct.Bitboard.html#method.shift_xy) methods. These methods shift all
//! squares in a `Bitboard` by a specified amount. The caveat with these methods is that they will
//! wrap around the y axis (eg. from a8 to b1 when shifting in the +y direction). Given that no
//! pawns should ever be on ranks 1 or 8, that issue can be easily avoided. These methods won't
//! wrap along the x axis, however, as seen in the second example below.
//!
//! The following example demonstrates how to compute non-capture pawn advancements. The following
//! code does not account for blocked pawns.
//!
//! ```rust
//! use chess::Square;
//! use chess::bitboard::Bitboard;
//!
//! let forward = -1; // black's turn, for white this would be 1
//! let pawns = Bitboard::from(Square::A7) | Square::B2.into();
//! let mut destinations = pawns.shift_y(forward);
//! assert_eq!(destinations.pop(), Some(Square::A6));
//! assert_eq!(destinations.pop(), Some(Square::B1));
//! assert_eq!(destinations.pop(), None);
//! ```
//!
//! The next example demonstrates how to compute pawn attacks:
//!
//! ```rust
//! use chess::Square;
//! use chess::bitboard::Bitboard;
//!
//! let forward = -1; // black's turn, for white this would be 1
//! let pawns = Bitboard::from(Square::A7) | Square::B2.into();
//!
//! // attacks toward king-side
//! let mut ks_attacks = pawns.shift_xy(1, forward);
//! assert_eq!(ks_attacks.pop(), Some(Square::B6));
//! assert_eq!(ks_attacks.pop(), Some(Square::C1));
//! assert_eq!(ks_attacks.pop(), None);
//!
//! // attacks toward queen side
//! // Note that since the pawn on a7 is on the far queen-side edge of the board, it
//! // has no attacks on that side. shift_xy handles this properly, without wrapping.
//! let mut qs_attacks = pawns.shift_xy(-1, forward);
//! assert_eq!(qs_attacks.pop(), Some(Square::A1));
//! assert_eq!(qs_attacks.pop(), None);
//! ```
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::convert::TryInto;
use std::iter::FusedIterator;
use std::iter::{FromIterator, Extend};
use std::ops;
use std::fmt;
use super::*;

mod attacks;
pub use attacks::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A set of squares with each bit representing one square
///
/// A `Bitboard` is, essentially, a set of [`Square`](../enum.Square.html)s stored in a 64-bit
/// integer. Each bit corresponds to one `Square`. If the bit is set, that `Square` is present. If
/// it is clear, the `Square` is not present. The diagram below shows the layout of the bits.
///
/// ```text
///     1    2    3    4    5    6    7    8
///    ---------------------------------------
/// h | 07 | 15 | 23 | 31 | 39 | 47 | 55 | 63 | h
///    ---------------------------------------
/// g | 06 | 14 | 22 | 30 | 38 | 46 | 54 | 62 | g
///    ---------------------------------------
/// f | 05 | 13 | 21 | 29 | 37 | 45 | 53 | 61 | f
///    ---------------------------------------
/// e | 04 | 12 | 20 | 28 | 36 | 44 | 52 | 60 | e
///    ---------------------------------------
/// d | 03 | 11 | 19 | 27 | 35 | 43 | 51 | 59 | d
///    ---------------------------------------
/// c | 02 | 10 | 18 | 26 | 34 | 42 | 50 | 58 | c
///    ---------------------------------------
/// b | 01 | 09 | 17 | 25 | 33 | 41 | 49 | 57 | b
///    ---------------------------------------
/// a | 00 | 08 | 16 | 24 | 32 | 40 | 48 | 56 | a
///    ---------------------------------------
///     1    2    3    4    5    6    7    8
/// ```
///
/// `Bitboard` implements all the bit-wise logic operators: `|`, `&`, `^`, `!`, `|=`, `&=`, and
/// `^=`. It also has methods that are typical for sets and collections, such as `insert`, `remove`,
/// `len`, and `contains`. It implements IntoIterator. However, since it's only a 64-bit value, it
/// implement's `Copy`, and there's no need for the borrowing iterator methods `iter` and
/// `iter_mut`.
///
/// The bit-shift operators are not implemented as they wouldn't be well-defined for a
/// 2-dimensional `Bitboard`. Instead, the methods, `shift_x`, `shift_y` and `shift_xy` are
/// provided. See the crate-level documentation for
/// [examples](index.html#pawn-advancements-and-attacks) of these methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Bitboard(u64);

impl Bitboard {
    /// Creates a new, empty bitboard
    pub fn new() -> Bitboard {
        Default::default()
    }

    /// Returns the number of squares in the bitboard
    pub fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns `true` if the bitboard is empty
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if the bitboard contains `sq`
    pub fn contains(self, sq: Square) -> bool {
        !(self & sq.into()).is_empty()
    }

    /// Returns `true` if `self` intersects `other`
    pub fn intersects(self, other: Bitboard) -> bool {
        !(self & other).is_empty()
    }

    /// Returns `true` if `self` does not intersect `other`
    pub fn is_disjoint(self, other: Bitboard) -> bool {
        (self & other).is_empty()
    }

    /// Adds a square to the bitboard if it is not already present
    pub fn insert(&mut self, sq: Square) {
        *self |= sq.into();
    }

    /// Removes a square from the bitboard if it is present
    pub fn remove(&mut self, sq: Square) {
        *self &= !Bitboard::from(sq);
    }

    /// Removes a square from the bitboard and returns it
    pub fn pop(&mut self) -> Option<Square> {
        if self.0 > 0 {
            // get the least significant bit
            let sq: Square = (self.0.trailing_zeros() as usize).try_into().expect("INFALLIBLE");
            // clear the least significant bit
            self.0 &= self.0 - 1;

            Some(sq)
        } else {
            None
        }
    }

    /// Returns the square that would be removed by a pop command
    pub fn peek(self) -> Option<Square> {
        if self.0 > 0 {
            // get the least significant bit
            Some((self.0.trailing_zeros() as usize).try_into().expect("INFALLIBLE"))
        } else {
            None
        }
    }

    /// Toggles a square in the bitboard
    pub fn toggle(&mut self, sq: Square) {
        *self ^= sq.into();
    }

    /// Returns a bitboard with all squares shifted by `x` files
    ///
    /// Overflow does not wrap.
    pub fn shift_x(self, x: i8) -> Bitboard {
        let bits = x << 3;

        if bits > 0 {
            Bitboard(self.0 << bits)
        } else {
            Bitboard(self.0 >> -bits)
        }
    }

    /// Returns a bitboard with all squares shifted by `y` ranks
    ///
    /// Rank overflow wraps to the next file, as seen below.
    ///
    /// ```rust
    /// # use chess::Square;
    /// # use chess::bitboard::Bitboard;
    /// #
    /// assert_eq!(Bitboard::from(Square::A8).shift_y(1), Bitboard::from(Square::B1));
    /// assert_eq!(Bitboard::from(Square::B1).shift_y(-1), Bitboard::from(Square::A8));
    /// ```
    ///
    /// See the crate-level documentation for
    /// [another example](index.html#pawn-advancements-and-attacks) of this method.
    pub fn shift_y(self, y: i8) -> Bitboard {
        let bits = y;

        if bits > 0 {
            Bitboard(self.0 << bits)
        } else {
            Bitboard(self.0 >> -bits)
        }
    }

    /// Returns a bitboard with all squares shifted by `x` files and `y` ranks.
    ///
    /// Rank overflow wraps to the next file. See the documentation for
    /// [`shift_y`](#method.shift_y). File overflow does not wrap.
    ///
    /// See the crate-level documentation for
    /// [an example](index.html#pawn-advancements-and-attacks) of this method.
    pub fn shift_xy(self, x:i8, y:i8) -> Bitboard {
        let bits = (x << 3) + y;

        if bits > 0 {
            Bitboard(self.0 << bits)
        } else {
            Bitboard(self.0 >> -bits)
        }
    }

    /// Returns a bitboard flipped horizontaly, so `File::A` becomes `File::H`
    pub fn swap_files(self) -> Bitboard {
        Bitboard(self.0.swap_bytes())
    }
}

impl ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Binary for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for Bitboard {
    fn from(val: u64) -> Bitboard {
        Bitboard(val)
    }
}

impl From<Square> for Bitboard {
    fn from(sq: Square) -> Bitboard {
        Bitboard(1 << sq as u64)
    }
}

impl From<File> for Bitboard {
    fn from(f: File) -> Bitboard {
        Bitboard(0x0000_0000_0000_00ff << (8 * f as u64))
    }
}

impl From<Rank> for Bitboard {
    fn from(r: Rank) -> Bitboard {
        Bitboard(0x0101_0101_0101_0101 << r as u64)
    }
}

impl From<IntoIter> for Bitboard {
    fn from(iter: IntoIter) -> Bitboard {
        iter.0
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl FromIterator<Square> for Bitboard {
    /// If converting from `bitboard::IntoIter`, use `Bitboard::from()` instead as that is faster
    fn from_iter<I: IntoIterator<Item=Square>>(iter: I) -> Self {
        let mut bd = Bitboard::new();

        for sq in iter {
            bd.insert(sq);
        }

        bd
    }
}

impl Extend<Square> for Bitboard {
    fn extend<I: IntoIterator<Item=Square>>(&mut self, iter: I) {
        for sq in iter {
            self.insert(sq);
        }
    }
}

/// Iterator over the squares of a `Bitboard`
#[derive(Debug, Copy, Clone)]
pub struct IntoIter(Bitboard);

impl Iterator for IntoIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IntoIter { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard() {
        // test new() method and Default trait
        assert_eq!(Bitboard::new(), Bitboard(0));
        assert_eq!(Bitboard::new(), Default::default());

        // test len() and is_empty() methods
        assert_eq!(Bitboard::new().len(), 0);
        assert!(Bitboard::new().is_empty());
        assert_eq!(Bitboard(0xffffffffffffffff).len(), 64);
        assert!(!Bitboard(0xffffffffffffffff).is_empty());

        // test contains() method
        assert!(Bitboard::from(Square::A1).contains(Square::A1));
        assert!(Bitboard::from(Square::H8).contains(Square::H8));
        assert!(!Bitboard::from(Square::A1).contains(Square::H8));
        assert!(!Bitboard::from(Square::H8).contains(Square::A1));

        // test intersects() and is_disjoint() methods

        // test insert(), remove() and toggle() methods

        // test formatting
        assert_eq!(format!("{}", Bitboard::from(0x0123456789abcdef)), "123456789abcdef");
        assert_eq!(format!("{:016}", Bitboard::from(0x0123456789abcdef)), "0123456789abcdef");
        assert_eq!(format!("{:x}", Bitboard::from(0x0123456789abcdef)), "123456789abcdef");
        assert_eq!(format!("{:016x}", Bitboard::from(0x0123456789abcdef)), "0123456789abcdef");
        assert_eq!(format!("{:X}", Bitboard::from(0x0123456789ABCDEF)), "123456789ABCDEF");
        assert_eq!(format!("{:016X}", Bitboard::from(0x0123456789ABCDEF)), "0123456789ABCDEF");
        assert_eq!(
            format!("{:o}", Bitboard::from(0x0123456789abcdef)),
            "4432126361152746757"
        );
        assert_eq!(
            format!("{:022o}", Bitboard::from(0x0123456789abcdef)),
            "0004432126361152746757"
        );
        assert_eq!(
            format!("{:b}", Bitboard::from(0x0123456789abcdef)),
            "100100011010001010110011110001001101010111100110111101111"
        );
        assert_eq!(
            format!("{:064b}", Bitboard::from(0x0123456789abcdef)),
            "0000000100100011010001010110011110001001101010111100110111101111"
        );
    }
}
