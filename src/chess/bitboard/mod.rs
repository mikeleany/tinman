//! Provides a representation of the pieces on the board
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
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
    /// Rank overflow wraps to the next or previous file.
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
    /// Rank overflow wraps to the next or previous file.
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
