//! Provides data and functions used to compute attacks
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use super::*;

const KING_ATTACKS: [Bitboard; Square::COUNT] = [
    Bitboard(0x0000_0000_0000_0302), Bitboard(0x0000_0000_0000_0705),
    Bitboard(0x0000_0000_0000_0e0a), Bitboard(0x0000_0000_0000_1c14),
    Bitboard(0x0000_0000_0000_3828), Bitboard(0x0000_0000_0000_7050),
    Bitboard(0x0000_0000_0000_e0a0), Bitboard(0x0000_0000_0000_c040),
    Bitboard(0x0000_0000_0003_0203), Bitboard(0x0000_0000_0007_0507),
    Bitboard(0x0000_0000_000e_0a0e), Bitboard(0x0000_0000_001c_141c),
    Bitboard(0x0000_0000_0038_2838), Bitboard(0x0000_0000_0070_5070),
    Bitboard(0x0000_0000_00e0_a0e0), Bitboard(0x0000_0000_00c0_40c0),
    Bitboard(0x0000_0000_0302_0300), Bitboard(0x0000_0000_0705_0700),
    Bitboard(0x0000_0000_0e0a_0e00), Bitboard(0x0000_0000_1c14_1c00),
    Bitboard(0x0000_0000_3828_3800), Bitboard(0x0000_0000_7050_7000),
    Bitboard(0x0000_0000_e0a0_e000), Bitboard(0x0000_0000_c040_c000),
    Bitboard(0x0000_0003_0203_0000), Bitboard(0x0000_0007_0507_0000),
    Bitboard(0x0000_000e_0a0e_0000), Bitboard(0x0000_001c_141c_0000),
    Bitboard(0x0000_0038_2838_0000), Bitboard(0x0000_0070_5070_0000),
    Bitboard(0x0000_00e0_a0e0_0000), Bitboard(0x0000_00c0_40c0_0000),
    Bitboard(0x0000_0302_0300_0000), Bitboard(0x0000_0705_0700_0000),
    Bitboard(0x0000_0e0a_0e00_0000), Bitboard(0x0000_1c14_1c00_0000),
    Bitboard(0x0000_3828_3800_0000), Bitboard(0x0000_7050_7000_0000),
    Bitboard(0x0000_e0a0_e000_0000), Bitboard(0x0000_c040_c000_0000),
    Bitboard(0x0003_0203_0000_0000), Bitboard(0x0007_0507_0000_0000),
    Bitboard(0x000e_0a0e_0000_0000), Bitboard(0x001c_141c_0000_0000),
    Bitboard(0x0038_2838_0000_0000), Bitboard(0x0070_5070_0000_0000),
    Bitboard(0x00e0_a0e0_0000_0000), Bitboard(0x00c0_40c0_0000_0000),
    Bitboard(0x0302_0300_0000_0000), Bitboard(0x0705_0700_0000_0000),
    Bitboard(0x0e0a_0e00_0000_0000), Bitboard(0x1c14_1c00_0000_0000),
    Bitboard(0x3828_3800_0000_0000), Bitboard(0x7050_7000_0000_0000),
    Bitboard(0xe0a0_e000_0000_0000), Bitboard(0xc040_c000_0000_0000),
    Bitboard(0x0203_0000_0000_0000), Bitboard(0x0507_0000_0000_0000),
    Bitboard(0x0a0e_0000_0000_0000), Bitboard(0x141c_0000_0000_0000),
    Bitboard(0x2838_0000_0000_0000), Bitboard(0x5070_0000_0000_0000),
    Bitboard(0xa0e0_0000_0000_0000), Bitboard(0x40c0_0000_0000_0000),
];

const KNIGHT_ATTACKS: [Bitboard; Square::COUNT] = [
    Bitboard(0x0000_0000_0002_0400), Bitboard(0x0000_0000_0005_0800),
    Bitboard(0x0000_0000_000a_1100), Bitboard(0x0000_0000_0014_2200),
    Bitboard(0x0000_0000_0028_4400), Bitboard(0x0000_0000_0050_8800),
    Bitboard(0x0000_0000_00a0_1000), Bitboard(0x0000_0000_0040_2000),
    Bitboard(0x0000_0000_0204_0004), Bitboard(0x0000_0000_0508_0008),
    Bitboard(0x0000_0000_0a11_0011), Bitboard(0x0000_0000_1422_0022),
    Bitboard(0x0000_0000_2844_0044), Bitboard(0x0000_0000_5088_0088),
    Bitboard(0x0000_0000_a010_0010), Bitboard(0x0000_0000_4020_0020),
    Bitboard(0x0000_0002_0400_0402), Bitboard(0x0000_0005_0800_0805),
    Bitboard(0x0000_000a_1100_110a), Bitboard(0x0000_0014_2200_2214),
    Bitboard(0x0000_0028_4400_4428), Bitboard(0x0000_0050_8800_8850),
    Bitboard(0x0000_00a0_1000_10a0), Bitboard(0x0000_0040_2000_2040),
    Bitboard(0x0000_0204_0004_0200), Bitboard(0x0000_0508_0008_0500),
    Bitboard(0x0000_0a11_0011_0a00), Bitboard(0x0000_1422_0022_1400),
    Bitboard(0x0000_2844_0044_2800), Bitboard(0x0000_5088_0088_5000),
    Bitboard(0x0000_a010_0010_a000), Bitboard(0x0000_4020_0020_4000),
    Bitboard(0x0002_0400_0402_0000), Bitboard(0x0005_0800_0805_0000),
    Bitboard(0x000a_1100_110a_0000), Bitboard(0x0014_2200_2214_0000),
    Bitboard(0x0028_4400_4428_0000), Bitboard(0x0050_8800_8850_0000),
    Bitboard(0x00a0_1000_10a0_0000), Bitboard(0x0040_2000_2040_0000),
    Bitboard(0x0204_0004_0200_0000), Bitboard(0x0508_0008_0500_0000),
    Bitboard(0x0a11_0011_0a00_0000), Bitboard(0x1422_0022_1400_0000),
    Bitboard(0x2844_0044_2800_0000), Bitboard(0x5088_0088_5000_0000),
    Bitboard(0xa010_0010_a000_0000), Bitboard(0x4020_0020_4000_0000),
    Bitboard(0x0400_0402_0000_0000), Bitboard(0x0800_0805_0000_0000),
    Bitboard(0x1100_110a_0000_0000), Bitboard(0x2200_2214_0000_0000),
    Bitboard(0x4400_4428_0000_0000), Bitboard(0x8800_8850_0000_0000),
    Bitboard(0x1000_10a0_0000_0000), Bitboard(0x2000_2040_0000_0000),
    Bitboard(0x0004_0200_0000_0000), Bitboard(0x0008_0500_0000_0000),
    Bitboard(0x0011_0a00_0000_0000), Bitboard(0x0022_1400_0000_0000),
    Bitboard(0x0044_2800_0000_0000), Bitboard(0x0088_5000_0000_0000),
    Bitboard(0x0010_a000_0000_0000), Bitboard(0x0020_4000_0000_0000),
];

const DIAGONAL_MASK: [Bitboard; Square::COUNT] = [
    Bitboard(0x8040_2010_0804_0201), Bitboard(0x0080_4020_1008_0402),
    Bitboard(0x0000_8040_2010_0804), Bitboard(0x0000_0080_4020_1008),
    Bitboard(0x0000_0000_8040_2010), Bitboard(0x0000_0000_0080_4020),
    Bitboard(0x0000_0000_0000_8040), Bitboard(0x0000_0000_0000_0080),
    Bitboard(0x4020_1008_0402_0100), Bitboard(0x8040_2010_0804_0201),
    Bitboard(0x0080_4020_1008_0402), Bitboard(0x0000_8040_2010_0804),
    Bitboard(0x0000_0080_4020_1008), Bitboard(0x0000_0000_8040_2010),
    Bitboard(0x0000_0000_0080_4020), Bitboard(0x0000_0000_0000_8040),
    Bitboard(0x2010_0804_0201_0000), Bitboard(0x4020_1008_0402_0100),
    Bitboard(0x8040_2010_0804_0201), Bitboard(0x0080_4020_1008_0402),
    Bitboard(0x0000_8040_2010_0804), Bitboard(0x0000_0080_4020_1008),
    Bitboard(0x0000_0000_8040_2010), Bitboard(0x0000_0000_0080_4020),
    Bitboard(0x1008_0402_0100_0000), Bitboard(0x2010_0804_0201_0000),
    Bitboard(0x4020_1008_0402_0100), Bitboard(0x8040_2010_0804_0201),
    Bitboard(0x0080_4020_1008_0402), Bitboard(0x0000_8040_2010_0804),
    Bitboard(0x0000_0080_4020_1008), Bitboard(0x0000_0000_8040_2010),
    Bitboard(0x0804_0201_0000_0000), Bitboard(0x1008_0402_0100_0000),
    Bitboard(0x2010_0804_0201_0000), Bitboard(0x4020_1008_0402_0100),
    Bitboard(0x8040_2010_0804_0201), Bitboard(0x0080_4020_1008_0402),
    Bitboard(0x0000_8040_2010_0804), Bitboard(0x0000_0080_4020_1008),
    Bitboard(0x0402_0100_0000_0000), Bitboard(0x0804_0201_0000_0000),
    Bitboard(0x1008_0402_0100_0000), Bitboard(0x2010_0804_0201_0000),
    Bitboard(0x4020_1008_0402_0100), Bitboard(0x8040_2010_0804_0201),
    Bitboard(0x0080_4020_1008_0402), Bitboard(0x0000_8040_2010_0804),
    Bitboard(0x0201_0000_0000_0000), Bitboard(0x0402_0100_0000_0000),
    Bitboard(0x0804_0201_0000_0000), Bitboard(0x1008_0402_0100_0000),
    Bitboard(0x2010_0804_0201_0000), Bitboard(0x4020_1008_0402_0100),
    Bitboard(0x8040_2010_0804_0201), Bitboard(0x0080_4020_1008_0402),
    Bitboard(0x0100_0000_0000_0000), Bitboard(0x0201_0000_0000_0000),
    Bitboard(0x0402_0100_0000_0000), Bitboard(0x0804_0201_0000_0000),
    Bitboard(0x1008_0402_0100_0000), Bitboard(0x2010_0804_0201_0000),
    Bitboard(0x4020_1008_0402_0100), Bitboard(0x8040_2010_0804_0201),
];

const ANTIDIAG_MASK: [Bitboard; Square::COUNT] = [
    Bitboard(0x0000_0000_0000_0001), Bitboard(0x0000_0000_0000_0102),
    Bitboard(0x0000_0000_0001_0204), Bitboard(0x0000_0000_0102_0408),
    Bitboard(0x0000_0001_0204_0810), Bitboard(0x0000_0102_0408_1020),
    Bitboard(0x0001_0204_0810_2040), Bitboard(0x0102_0408_1020_4080),
    Bitboard(0x0000_0000_0000_0102), Bitboard(0x0000_0000_0001_0204),
    Bitboard(0x0000_0000_0102_0408), Bitboard(0x0000_0001_0204_0810),
    Bitboard(0x0000_0102_0408_1020), Bitboard(0x0001_0204_0810_2040),
    Bitboard(0x0102_0408_1020_4080), Bitboard(0x0204_0810_2040_8000),
    Bitboard(0x0000_0000_0001_0204), Bitboard(0x0000_0000_0102_0408),
    Bitboard(0x0000_0001_0204_0810), Bitboard(0x0000_0102_0408_1020),
    Bitboard(0x0001_0204_0810_2040), Bitboard(0x0102_0408_1020_4080),
    Bitboard(0x0204_0810_2040_8000), Bitboard(0x0408_1020_4080_0000),
    Bitboard(0x0000_0000_0102_0408), Bitboard(0x0000_0001_0204_0810),
    Bitboard(0x0000_0102_0408_1020), Bitboard(0x0001_0204_0810_2040),
    Bitboard(0x0102_0408_1020_4080), Bitboard(0x0204_0810_2040_8000),
    Bitboard(0x0408_1020_4080_0000), Bitboard(0x0810_2040_8000_0000),
    Bitboard(0x0000_0001_0204_0810), Bitboard(0x0000_0102_0408_1020),
    Bitboard(0x0001_0204_0810_2040), Bitboard(0x0102_0408_1020_4080),
    Bitboard(0x0204_0810_2040_8000), Bitboard(0x0408_1020_4080_0000),
    Bitboard(0x0810_2040_8000_0000), Bitboard(0x1020_4080_0000_0000),
    Bitboard(0x0000_0102_0408_1020), Bitboard(0x0001_0204_0810_2040),
    Bitboard(0x0102_0408_1020_4080), Bitboard(0x0204_0810_2040_8000),
    Bitboard(0x0408_1020_4080_0000), Bitboard(0x0810_2040_8000_0000),
    Bitboard(0x1020_4080_0000_0000), Bitboard(0x2040_8000_0000_0000),
    Bitboard(0x0001_0204_0810_2040), Bitboard(0x0102_0408_1020_4080),
    Bitboard(0x0204_0810_2040_8000), Bitboard(0x0408_1020_4080_0000),
    Bitboard(0x0810_2040_8000_0000), Bitboard(0x1020_4080_0000_0000),
    Bitboard(0x2040_8000_0000_0000), Bitboard(0x4080_0000_0000_0000),
    Bitboard(0x0102_0408_1020_4080), Bitboard(0x0204_0810_2040_8000),
    Bitboard(0x0408_1020_4080_0000), Bitboard(0x0810_2040_8000_0000),
    Bitboard(0x1020_4080_0000_0000), Bitboard(0x2040_8000_0000_0000),
    Bitboard(0x4080_0000_0000_0000), Bitboard(0x8000_0000_0000_0000),
];

const FILE_ATTACKS: [[Bitboard; 64]; Rank::COUNT] = [
    [
        Bitboard(0x0000_0000_0000_00fe), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_001e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_003e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_001e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_007e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_001e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_003e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_001e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_000e), Bitboard(0x0000_0000_0000_0002),
        Bitboard(0x0000_0000_0000_0006), Bitboard(0x0000_0000_0000_0002),
    ],
    [
        Bitboard(0x0000_0000_0000_00fd), Bitboard(0x0000_0000_0000_00fd),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_001d), Bitboard(0x0000_0000_0000_001d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_003d), Bitboard(0x0000_0000_0000_003d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_001d), Bitboard(0x0000_0000_0000_001d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_007d), Bitboard(0x0000_0000_0000_007d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_001d), Bitboard(0x0000_0000_0000_001d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_003d), Bitboard(0x0000_0000_0000_003d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_001d), Bitboard(0x0000_0000_0000_001d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
        Bitboard(0x0000_0000_0000_000d), Bitboard(0x0000_0000_0000_000d),
        Bitboard(0x0000_0000_0000_0005), Bitboard(0x0000_0000_0000_0005),
    ],
    [
        Bitboard(0x0000_0000_0000_00fb), Bitboard(0x0000_0000_0000_00fa),
        Bitboard(0x0000_0000_0000_00fb), Bitboard(0x0000_0000_0000_00fa),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_003b), Bitboard(0x0000_0000_0000_003a),
        Bitboard(0x0000_0000_0000_003b), Bitboard(0x0000_0000_0000_003a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_007b), Bitboard(0x0000_0000_0000_007a),
        Bitboard(0x0000_0000_0000_007b), Bitboard(0x0000_0000_0000_007a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_003b), Bitboard(0x0000_0000_0000_003a),
        Bitboard(0x0000_0000_0000_003b), Bitboard(0x0000_0000_0000_003a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_001b), Bitboard(0x0000_0000_0000_001a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
        Bitboard(0x0000_0000_0000_000b), Bitboard(0x0000_0000_0000_000a),
    ],
    [
        Bitboard(0x0000_0000_0000_00f7), Bitboard(0x0000_0000_0000_00f6),
        Bitboard(0x0000_0000_0000_00f4), Bitboard(0x0000_0000_0000_00f4),
        Bitboard(0x0000_0000_0000_00f7), Bitboard(0x0000_0000_0000_00f6),
        Bitboard(0x0000_0000_0000_00f4), Bitboard(0x0000_0000_0000_00f4),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0037), Bitboard(0x0000_0000_0000_0036),
        Bitboard(0x0000_0000_0000_0034), Bitboard(0x0000_0000_0000_0034),
        Bitboard(0x0000_0000_0000_0037), Bitboard(0x0000_0000_0000_0036),
        Bitboard(0x0000_0000_0000_0034), Bitboard(0x0000_0000_0000_0034),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0077), Bitboard(0x0000_0000_0000_0076),
        Bitboard(0x0000_0000_0000_0074), Bitboard(0x0000_0000_0000_0074),
        Bitboard(0x0000_0000_0000_0077), Bitboard(0x0000_0000_0000_0076),
        Bitboard(0x0000_0000_0000_0074), Bitboard(0x0000_0000_0000_0074),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0037), Bitboard(0x0000_0000_0000_0036),
        Bitboard(0x0000_0000_0000_0034), Bitboard(0x0000_0000_0000_0034),
        Bitboard(0x0000_0000_0000_0037), Bitboard(0x0000_0000_0000_0036),
        Bitboard(0x0000_0000_0000_0034), Bitboard(0x0000_0000_0000_0034),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
        Bitboard(0x0000_0000_0000_0017), Bitboard(0x0000_0000_0000_0016),
        Bitboard(0x0000_0000_0000_0014), Bitboard(0x0000_0000_0000_0014),
    ],
    [
        Bitboard(0x0000_0000_0000_00ef), Bitboard(0x0000_0000_0000_00ee),
        Bitboard(0x0000_0000_0000_00ec), Bitboard(0x0000_0000_0000_00ec),
        Bitboard(0x0000_0000_0000_00e8), Bitboard(0x0000_0000_0000_00e8),
        Bitboard(0x0000_0000_0000_00e8), Bitboard(0x0000_0000_0000_00e8),
        Bitboard(0x0000_0000_0000_00ef), Bitboard(0x0000_0000_0000_00ee),
        Bitboard(0x0000_0000_0000_00ec), Bitboard(0x0000_0000_0000_00ec),
        Bitboard(0x0000_0000_0000_00e8), Bitboard(0x0000_0000_0000_00e8),
        Bitboard(0x0000_0000_0000_00e8), Bitboard(0x0000_0000_0000_00e8),
        Bitboard(0x0000_0000_0000_002f), Bitboard(0x0000_0000_0000_002e),
        Bitboard(0x0000_0000_0000_002c), Bitboard(0x0000_0000_0000_002c),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_002f), Bitboard(0x0000_0000_0000_002e),
        Bitboard(0x0000_0000_0000_002c), Bitboard(0x0000_0000_0000_002c),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_006f), Bitboard(0x0000_0000_0000_006e),
        Bitboard(0x0000_0000_0000_006c), Bitboard(0x0000_0000_0000_006c),
        Bitboard(0x0000_0000_0000_0068), Bitboard(0x0000_0000_0000_0068),
        Bitboard(0x0000_0000_0000_0068), Bitboard(0x0000_0000_0000_0068),
        Bitboard(0x0000_0000_0000_006f), Bitboard(0x0000_0000_0000_006e),
        Bitboard(0x0000_0000_0000_006c), Bitboard(0x0000_0000_0000_006c),
        Bitboard(0x0000_0000_0000_0068), Bitboard(0x0000_0000_0000_0068),
        Bitboard(0x0000_0000_0000_0068), Bitboard(0x0000_0000_0000_0068),
        Bitboard(0x0000_0000_0000_002f), Bitboard(0x0000_0000_0000_002e),
        Bitboard(0x0000_0000_0000_002c), Bitboard(0x0000_0000_0000_002c),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_002f), Bitboard(0x0000_0000_0000_002e),
        Bitboard(0x0000_0000_0000_002c), Bitboard(0x0000_0000_0000_002c),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
        Bitboard(0x0000_0000_0000_0028), Bitboard(0x0000_0000_0000_0028),
    ],
    [
        Bitboard(0x0000_0000_0000_00df), Bitboard(0x0000_0000_0000_00de),
        Bitboard(0x0000_0000_0000_00dc), Bitboard(0x0000_0000_0000_00dc),
        Bitboard(0x0000_0000_0000_00d8), Bitboard(0x0000_0000_0000_00d8),
        Bitboard(0x0000_0000_0000_00d8), Bitboard(0x0000_0000_0000_00d8),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00df), Bitboard(0x0000_0000_0000_00de),
        Bitboard(0x0000_0000_0000_00dc), Bitboard(0x0000_0000_0000_00dc),
        Bitboard(0x0000_0000_0000_00d8), Bitboard(0x0000_0000_0000_00d8),
        Bitboard(0x0000_0000_0000_00d8), Bitboard(0x0000_0000_0000_00d8),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_00d0), Bitboard(0x0000_0000_0000_00d0),
        Bitboard(0x0000_0000_0000_005f), Bitboard(0x0000_0000_0000_005e),
        Bitboard(0x0000_0000_0000_005c), Bitboard(0x0000_0000_0000_005c),
        Bitboard(0x0000_0000_0000_0058), Bitboard(0x0000_0000_0000_0058),
        Bitboard(0x0000_0000_0000_0058), Bitboard(0x0000_0000_0000_0058),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_005f), Bitboard(0x0000_0000_0000_005e),
        Bitboard(0x0000_0000_0000_005c), Bitboard(0x0000_0000_0000_005c),
        Bitboard(0x0000_0000_0000_0058), Bitboard(0x0000_0000_0000_0058),
        Bitboard(0x0000_0000_0000_0058), Bitboard(0x0000_0000_0000_0058),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
        Bitboard(0x0000_0000_0000_0050), Bitboard(0x0000_0000_0000_0050),
    ],
    [
        Bitboard(0x0000_0000_0000_00bf), Bitboard(0x0000_0000_0000_00be),
        Bitboard(0x0000_0000_0000_00bc), Bitboard(0x0000_0000_0000_00bc),
        Bitboard(0x0000_0000_0000_00b8), Bitboard(0x0000_0000_0000_00b8),
        Bitboard(0x0000_0000_0000_00b8), Bitboard(0x0000_0000_0000_00b8),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00bf), Bitboard(0x0000_0000_0000_00be),
        Bitboard(0x0000_0000_0000_00bc), Bitboard(0x0000_0000_0000_00bc),
        Bitboard(0x0000_0000_0000_00b8), Bitboard(0x0000_0000_0000_00b8),
        Bitboard(0x0000_0000_0000_00b8), Bitboard(0x0000_0000_0000_00b8),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00b0), Bitboard(0x0000_0000_0000_00b0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
        Bitboard(0x0000_0000_0000_00a0), Bitboard(0x0000_0000_0000_00a0),
    ],
    [
        Bitboard(0x0000_0000_0000_007f), Bitboard(0x0000_0000_0000_007e),
        Bitboard(0x0000_0000_0000_007c), Bitboard(0x0000_0000_0000_007c),
        Bitboard(0x0000_0000_0000_0078), Bitboard(0x0000_0000_0000_0078),
        Bitboard(0x0000_0000_0000_0078), Bitboard(0x0000_0000_0000_0078),
        Bitboard(0x0000_0000_0000_0070), Bitboard(0x0000_0000_0000_0070),
        Bitboard(0x0000_0000_0000_0070), Bitboard(0x0000_0000_0000_0070),
        Bitboard(0x0000_0000_0000_0070), Bitboard(0x0000_0000_0000_0070),
        Bitboard(0x0000_0000_0000_0070), Bitboard(0x0000_0000_0000_0070),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0060), Bitboard(0x0000_0000_0000_0060),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
        Bitboard(0x0000_0000_0000_0040), Bitboard(0x0000_0000_0000_0040),
    ],
];

/// Computes sliding attacks along the rank of `sq` based on the occupied squares
/// given by `occ`
///
/// This function is similar to [`rook_attacks`](fn.rook_attacks.html), but only computes attacks
/// along a single rank. This function is useful for determining if the space is clear between the
/// king and a rook as required for castling.
///
/// ```rust
/// use chess::Square;
/// use chess::bitboard::{Bitboard, rank_attacks};
///
/// // squares occupied by white rooks
/// let rooks = Bitboard::from(Square::A1) | Square::H1.into();
/// // occupied squares (those on the first rank, anyway)
/// let occ = rooks | Square::D1.into() | Square::E1.into();
/// // rooks with no pieces between them and the king on e1
/// let mut visible_rooks = rank_attacks(Square::E1, occ) & rooks;
/// assert_eq!(visible_rooks.pop(), Some(Square::H1));
/// assert_eq!(visible_rooks.pop(), None);
/// ```
///
/// See also [Sliding Attacks (Bishops, Rooks and
/// Queens)](index.html#sliding-attacks-bishops-rooks-and-queens).
pub fn rank_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let sq_mask = Bitboard::from(sq).0;
    let rank_mask = Bitboard::from(sq.rank()).0;

    let masked = occ.0 & rank_mask.wrapping_sub(sq_mask);
    let mut rank_att = masked.wrapping_sub(sq_mask);
    rank_att ^= masked.swap_bytes().wrapping_sub(sq_mask.swap_bytes()).swap_bytes();
    rank_att &= rank_mask;

    Bitboard(rank_att)
}

/// Computes knight-like attacks to or from `sq`
///
/// See the crate-level documentation for more information about
/// [this function](index.html#direct-attacks-knights-and-kings) and
/// [other attack functions](index.html#moves-and-attacks).
#[inline]
pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq as usize]
}

/// Computes bishop-like attacks to or from `sq` based on the occupied squares
/// given by `occ`
///
/// See the crate-level documentation for more information about
/// [this function](index.html#sliding-attacks-bishops-rooks-and-queens) and
/// [other attack functions](index.html#moves-and-attacks).
pub fn bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let sq_mask = Bitboard::from(sq).0;
    let swapped = sq_mask.swap_bytes();

    let masked = occ.0 & DIAGONAL_MASK[sq as usize].0.wrapping_sub(sq_mask);
    let mut diag = masked.wrapping_sub(sq_mask);
    diag ^= masked.swap_bytes().wrapping_sub(swapped).swap_bytes();
    diag &= DIAGONAL_MASK[sq as usize].0;

    let masked = occ.0 & ANTIDIAG_MASK[sq as usize].0.wrapping_sub(sq_mask);
    let mut anti = masked.wrapping_sub(sq_mask);
    anti ^= masked.swap_bytes().wrapping_sub(swapped).swap_bytes();
    anti &= ANTIDIAG_MASK[sq as usize].0;

    Bitboard(diag | anti)
}

/// Computes rook-like attacks to or from `sq` based on the occupied squares
/// given by `occ`
///
/// See the crate-level documentation for more information about
/// [this function](index.html#sliding-attacks-bishops-rooks-and-queens) and
/// [other attack functions](index.html#moves-and-attacks).
pub fn rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let file_att = FILE_ATTACKS[sq.rank() as usize]
        [((occ.0 >> (sq.file() as usize * Rank::COUNT + 1)) & 0o77) as usize].0
        << (sq.file() as usize * Rank::COUNT);

    rank_attacks(sq, occ) | Bitboard(file_att)
}

/// Computes queen-like attacks to or from square based on the occupied squares
/// given by `occ`
///
/// See the crate-level documentation for more information about
/// [this function](index.html#sliding-attacks-bishops-rooks-and-queens) and
/// [other attack functions](index.html#moves-and-attacks).
#[inline]
pub fn queen_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

/// Computes king-like attacks to or from `sq`
///
/// See the crate-level documentation for more information about
/// [this function](index.html#direct-attacks-knights-and-kings) and
/// [other attack functions](index.html#moves-and-attacks).
#[inline]
pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq as usize]
}
