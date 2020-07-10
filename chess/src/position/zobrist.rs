//! Contains structure and data for Zobrist hash keys
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fmt;
use super::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A 64-bit hash key generated from a position
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Zobrist(u64);

impl Zobrist {
    /// Creates a new zobrist key
    pub fn new() -> Zobrist {
        Zobrist(0)
    }

    /// Toggles piece placement
    pub fn toggle_piece_placement(&mut self, c: Color, p: Piece, sq: Square) {
        self.0 ^= PIECE_PLACEMENT[c as usize][p as usize][sq as usize];
    }

    /// Toggles an en passant square
    pub fn toggle_ep_square(&mut self, sq: Square) {
        self.0 ^= EP_SQUARE[sq.file() as usize];
    }

    /// Toggles castling flags
    pub fn toggle_castling_rights(&mut self, c: Color, flags: u8) {
        self.0 ^= CASTLE_FLAGS[c as usize][flags as usize];
    }

    /// Toggles whose turn it is
    pub fn toggle_turn(&mut self) {
        self.0 ^= BLACK_MOVE;
    }
}

impl fmt::Display for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Binary for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Zobrist> for u64 {
    /// Allows using the key to get a hash table index
    ///
    /// # Example
    /// ```rust
    /// use chess::Position;
    ///
    /// let pos = Position::new();
    /// let hash_table_size: usize = 0x10_0000;
    /// let index = u64::from(pos.zobrist_key()) as usize & (hash_table_size - 1);
    /// ```
    fn from(key: Zobrist) -> Self {
        key.0
    }
}

const PIECE_PLACEMENT: [[[u64; Square::COUNT]; Piece::COUNT]; Color::COUNT] = [
    // white
    [
        // white pawn
        [
            0xcd19_de16_cd0e_60be, 0xabea_b225_f864_018d,
            0x38f5_e044_c27e_2919, 0xd8fe_af51_83e1_2a86,
            0xcf20_4fdd_3580_2379, 0xfbf6_a26e_dc97_2b1f,
            0xe65b_c844_6f5a_003d, 0x507d_f8bc_f2a8_565b,
            0xbdec_e26c_131c_29a4, 0x934d_267a_bfff_309a,
            0x6bc1_e2e4_c483_788e, 0x03fb_6da7_2606_f89a,
            0xa4cd_4338_4494_d409, 0x4fc0_2eb6_a872_abeb,
            0x758c_0e07_cb8b_ad7c, 0xa0e9_7fda_7b62_92a3,
            0x05ef_c190_0edc_015f, 0x27bc_07ac_3627_3895,
            0xf12c_709e_b159_c2d5, 0x72fd_2048_7f7a_9c47,
            0x8f1f_2604_d2c1_5e97, 0xd757_1830_32e6_4668,
            0xca4b_232b_1343_5002, 0xf652_fe81_d2b6_ee97,
            0xc2a0_6e4a_a6c1_9944, 0x2eb3_c857_94b2_4ca0,
            0x8a8c_4953_02dc_4de3, 0x614e_c566_fb68_b794,
            0xc113_96b2_4a10_452f, 0xfeb7_4a28_b8e2_3e60,
            0xa8db_e567_d573_7851, 0xc363_2e21_8afc_f7bd,
            0xa177_da12_25d1_8607, 0xe115_3d1b_dcb3_61a1,
            0x955b_eefb_1446_5445, 0x95aa_0262_3b2a_ca8f,
            0x043c_113d_5185_ce3d, 0x58ae_6555_1848_e971,
            0xcedc_7580_226e_4b9f, 0xda44_66df_e80d_fad5,
            0xce65_1b01_cfad_43fb, 0x0742_efab_6123_f4a9,
            0x8eee_d7d8_6d77_8f9b, 0x279c_071b_8c36_a3f0,
            0x39c4_759e_77a2_9d4b, 0xf577_fb69_5914_d1f6,
            0xd986_59fd_3db3_b2db, 0x3c77_fb6b_a3b3_e0f7,
            0xa748_72ca_419c_8453, 0x2a34_62de_ee8f_aa55,
            0xfd32_a1a1_7b3e_2c22, 0xf9d8_8244_9b04_cabd,
            0xe05e_af39_e9f5_d4c5, 0x0e52_fba6_138b_f2e6,
            0xd0e8_e2d7_fcb7_32ad, 0xdfb7_e5cd_2f0f_45bc,
            0xb469_b6a3_2096_4365, 0x599b_c7c3_0fbd_472d,
            0x3c63_c5bd_34a8_a531, 0x6991_26b2_e2fa_60e6,
            0x1f47_fd55_b9a1_40bc, 0x8621_d06e_3360_8cb3,
            0xd81b_4e2d_52ac_ec7d, 0xe7c9_453d_f105_4657,
        ],
        // white knight
        [
            0xb1c7_4e6f_2d42_0abe, 0xb2d8_41b2_5b54_feed,
            0xd63a_5c8a_2ce8_0aa7, 0xbefe_add4_97e2_1333,
            0x8280_2df2_8951_486e, 0xd55c_2cf0_01a8_89c0,
            0x1675_4fb7_0f30_ab4b, 0x7591_e764_569d_2973,
            0xe36a_0d09_946a_2fdc, 0x89f1_e258_e113_0884,
            0x326f_9136_9db4_7623, 0x44d8_9133_1123_b47d,
            0x4dfc_8ea6_a6b6_c864, 0x29e3_43ef_e29f_01cd,
            0xd24d_2688_e389_8acc, 0x3fe6_f6f7_48fe_3108,
            0xf4d5_d121_a197_4240, 0x69b2_11d1_dde2_fa32,
            0x4891_52ea_6524_26ba, 0xcd0b_71ba_3a23_f8cb,
            0xc22b_5eff_8008_8406, 0x006c_b5e1_b492_e55d,
            0x6430_6379_2570_01e5, 0x716b_230d_bcd4_03d8,
            0x08c9_20be_eda1_4ac4, 0x16d9_9cb9_ac02_2ff4,
            0x33bb_18e2_5411_6e9e, 0x0afd_2200_affc_fb5d,
            0xb4a2_98a3_f95d_cdc1, 0x1b7c_f4e7_6996_6373,
            0x269f_1794_59e2_4d79, 0xb5d1_12da_f102_becc,
            0x07fa_f18f_390c_16e5, 0xd868_ed68_3732_a4dd,
            0x3972_0774_389e_2671, 0xd9b4_6298_2608_bc5e,
            0x3223_a9d5_f27c_80c5, 0xe7e8_cc67_7492_8f5b,
            0x1a5f_4c18_72c8_f81a, 0x1d1c_9f33_dfa5_be07,
            0x4cca_e31a_1f55_0b4a, 0xe4ff_7745_5a9d_2bba,
            0x729b_7e82_f0be_8e07, 0x3a74_a5a2_d70f_0bc1,
            0x8cdb_a02c_60a8_780c, 0xf4b2_44de_75ac_05cd,
            0xc1f5_1165_9802_6f4a, 0xf4b6_9ea6_4dba_0445,
            0xb3fd_79ee_3737_5f41, 0x8535_331a_7bc1_a0b6,
            0x7979_c8ea_89bc_b621, 0xa857_035a_d36d_8d24,
            0x0f99_943c_1568_ae5c, 0x5fdd_d8b8_63b5_cb0d,
            0x4222_eef7_3d6d_61be, 0x358d_2285_f7c8_fb41,
            0x927c_dbd7_1ff8_4145, 0x6af2_ac67_e25a_93f5,
            0x9ca8_6cfe_eccd_f338, 0x40e4_dfaf_ac40_54b5,
            0xd60b_d654_b254_8744, 0xb84d_7a1c_b28f_e20c,
            0x4759_3450_05f2_66a1, 0x65f4_983e_944b_27ec,
        ],
        // white bishop
        [
            0x072b_a4fd_5b8e_0d13, 0x1008_8d0f_fbb3_647e,
            0xbdcd_1faf_4d4a_ed3f, 0x1711_9df0_9892_d93a,
            0x41b0_c0d1_fb91_9d12, 0x49f3_5c26_49dc_c125,
            0xd49e_e9be_f697_1c8d, 0x303a_aaef_f38e_6dd4,
            0xd7d4_a828_9ea8_9697, 0x66f6_ae6f_d6a2_7743,
            0x5c5d_29cf_4487_089d, 0xccea_4e47_e7a5_404b,
            0x5af2_d6c5_497a_162a, 0xc568_ef7d_2d15_902d,
            0x1eff_deb7_8515_a446, 0xfd0f_2940_645c_7930,
            0x899b_461b_123b_b2c9, 0x88ea_3ca0_7b66_029c,
            0xd29d_884c_a801_cbf8, 0x0daf_6350_6422_d28b,
            0x5720_b3f8_ce57_4d21, 0xda0b_9def_6680_6260,
            0xd60a_7f51_919e_bd02, 0x8441_868d_5c9e_1ecc,
            0x347d_31fb_d3a1_5f2d, 0xa890_d052_3453_32c5,
            0x8540_6aaa_fb90_35c4, 0xdb18_8990_a836_a2d6,
            0x9787_b6cc_91e9_ad32, 0xbd42_e991_61d6_cc93,
            0xc2a2_aada_4883_3fc1, 0x6bd9_6ff2_9dc9_5438,
            0xfd7e_aa11_0a10_792c, 0x2857_5f6a_f9b9_08b0,
            0x920d_d4b8_efd8_e498, 0x3517_2a37_4ba0_4c83,
            0xe5e7_a432_6786_998f, 0x5972_3aa4_40d1_2072,
            0xe2c8_9478_663a_4edd, 0x67f2_9e4f_0a8d_02c6,
            0x82c1_dad7_4145_1479, 0x6e2b_7f32_8d31_7486,
            0xe538_91e7_a839_17d8, 0xead3_0191_5e44_1c2d,
            0x1b0c_f333_473d_2043, 0x843a_124b_5cf9_2b85,
            0x8979_2efb_cbc8_c329, 0x3e39_ed3a_d7e1_e4c7,
            0x5ca1_330c_6130_aa73, 0x1c2b_a792_01e0_d929,
            0x03bd_399a_3db8_a951, 0x709c_d677_efad_c16d,
            0x1a66_4989_8406_c618, 0xd9ac_7e3b_7668_9f29,
            0x7387_e9ad_9028_d217, 0x0e63_d2ed_0b0b_2bcf,
            0x33cd_28c8_c299_a285, 0x2e6a_f73a_31ca_78b6,
            0x0ab9_c475_fee5_7fdf, 0x3aeb_d2c4_239f_09a3,
            0xa2aa_a930_5ee8_eb73, 0xbc88_4d6e_0f99_a19b,
            0x5172_3e24_0ebc_6cd7, 0x06ee_a3b4_ef59_8109,
        ],
        // white rook
        [
            0x5faa_b93e_c4ab_6ff9, 0xec60_ef8d_e0d0_b388,
            0x7da4_f8da_a014_4a08, 0x63c5_32b7_ecc9_fb0f,
            0x1e5b_fcc6_539b_23b3, 0x7361_cef3_7725_4a3c,
            0xfb75_85d6_7630_07f6, 0x4642_8f18_72b0_d877,
            0x7347_b78e_d8c6_6823, 0x65fe_9f58_2853_9e2a,
            0x26f9_e951_4854_b73a, 0xae51_c383_f4cf_d7bc,
            0x349e_51ac_bd69_ecca, 0x478d_4203_9762_0cbc,
            0x9b3a_9b0d_f8c6_b934, 0xdb63_d82e_3f66_ff0c,
            0x5be8_2dba_33ce_eb39, 0x50e9_4cb2_6279_c047,
            0x58a7_fee1_dad6_c527, 0x5df9_3d37_67ae_0db9,
            0xc680_00fd_3ea8_c61e, 0x253b_9489_5fd2_fd7f,
            0x45fc_e606_c9b0_ba32, 0xe97c_aed2_55e4_f9cb,
            0x5a2f_c00f_8cac_ba6d, 0x9821_91c5_3e1e_0155,
            0x03c6_315d_e5eb_23ee, 0x8de6_c6bf_2d50_266b,
            0xbad7_490a_9d52_73b1, 0x8223_5534_4d62_5039,
            0xdf73_ddc3_0eca_62b2, 0xf64b_1402_f736_5434,
            0xfe2a_343d_e3ce_9480, 0xe07b_086d_7548_97be,
            0xf2fe_0361_6c2e_a875, 0x8924_d1e2_35c2_8c70,
            0x0a84_745d_dd46_8414, 0xbd3b_31d0_9de0_83a5,
            0xbe27_9105_9e42_3b5c, 0x877f_9725_43e2_882d,
            0x8dcb_af43_4f2c_1203, 0x9cb9_3548_0fd8_2746,
            0xc645_cc7e_67d3_aee6, 0x87fb_9699_a719_5440,
            0xb96b_782a_22d8_c126, 0x6d58_cad3_9127_dc68,
            0x09aa_e0fc_e862_04c0, 0xea93_49da_b848_3327,
            0x3065_d46a_9352_ca96, 0x36a6_63cb_432d_b86d,
            0x5397_0471_baec_e637, 0x224b_9f5f_485f_fdcf,
            0xd46d_b3c6_a253_29da, 0x04c6_b7f4_8747_f8f4,
            0xbdd0_fcfb_8e5a_6759, 0xf1a3_04cb_2613_8440,
            0x5f25_452f_0970_432d, 0xcd34_f351_6ed3_08cf,
            0xecb1_0f50_0cb7_04aa, 0xf6e7_e682_a97a_af2d,
            0xf461_5116_1c8a_f4f5, 0x49de_3db9_7bb0_fe68,
            0xd2eb_9f2c_2608_cc8e, 0x254d_7d7f_aaa1_6365,
        ],
        // white queen
        [
            0x09c8_0ac3_13e5_e54b, 0x07ec_0476_8818_682b,
            0xc60f_c601_685c_1cd3, 0x24a4_340a_5d5e_de76,
            0x0b06_8a3d_aac3_e0b4, 0xeb85_ad2d_226c_d172,
            0x64e6_2d4e_56a7_3f60, 0x7d35_fbc8_b713_b892,
            0xd948_1c2e_cde9_8e78, 0x4c1e_ef28_f19b_aefb,
            0x172a_f6e1_8945_f428, 0x4163_9dc4_c401_363d,
            0xe41b_c816_1b27_3e12, 0x2374_088c_9975_0158,
            0x81cb_ac39_1394_cdd3, 0x3321_7a98_dd95_ce30,
            0x348a_bfe7_2339_a161, 0xf53f_ca4f_a1fd_35d4,
            0xe84b_79f8_d2e0_43c1, 0x0840_714d_8224_4636,
            0x2f4f_298a_c3de_c0e0, 0xb64e_08a6_38ae_aa3c,
            0x7a94_2ccc_b539_8d7d, 0xadd0_174e_d48f_1f47,
            0x798f_996b_78a3_b286, 0xeb6d_48bd_1fc1_f991,
            0x03e7_7bdf_9977_7148, 0x69da_1220_945f_830b,
            0xcd2f_8709_b766_044a, 0x9d2d_84ab_3061_1298,
            0x67d3_09c7_7d79_16cf, 0x1867_8bd9_9358_d3c1,
            0xc9bb_4489_ad8c_0b41, 0xfd75_1eb2_14d7_5954,
            0x9a4e_c4fb_ef8b_2f04, 0x00e3_fb63_ed79_3f82,
            0x0ee4_4250_f2e4_8873, 0xdbed_5cc0_1fa2_5460,
            0xffd3_90ce_f812_0217, 0x00ca_b97c_8872_f7f4,
            0x2d8e_a79a_0745_5441, 0x4638_0530_648b_a72b,
            0x62a2_5feb_f05d_349b, 0x08b6_1b7b_9f95_ef58,
            0x3c74_891e_afe2_0178, 0xf105_f5d5_78ae_5a22,
            0xea0c_2959_fbac_54c6, 0x28b5_0cdf_612f_3c01,
            0x1d60_46b0_894b_a90c, 0x46f8_ce48_7e6c_b1af,
            0xc0d4_60fe_1778_80d8, 0xa8f8_609a_e25f_832a,
            0xbff6_d8e6_6325_4f6d, 0x3a5f_0675_625f_152a,
            0x57a3_dca9_f7dc_b7ad, 0xebd8_5323_ee5b_0c35,
            0xef16_17c8_3a98_9488, 0x36be_0a72_6e36_c6a5,
            0x7097_4ea2_2540_a467, 0x162c_f650_7630_5d4e,
            0x75ce_437d_e86e_9f66, 0xcf33_3497_968f_6ca1,
            0x60de_beb8_1031_f646, 0x1a8b_023a_525e_2551,
        ],
        // white king
        [
            0xbaf1_7ff0_a5ea_826d, 0xa8e2_57ff_d7e9_7675,
            0x06ab_beda_28bd_aea3, 0x3565_aad8_1073_959b,
            0x074e_b425_e43d_5fbf, 0xa1e8_f4d2_5b49_3572,
            0x4ee5_0471_bd87_65b2, 0xa7a2_b829_843f_46ed,
            0x2be3_e48c_419c_b077, 0xb8f9_e1c0_730a_d1f5,
            0x9b57_acb5_95d7_21b1, 0x0f6d_1da6_84bc_8d2f,
            0x0070_3d21_1bc0_92f8, 0x09f2_799a_1cd5_5fa3,
            0x24d8_984b_c664_04a9, 0xbeec_3140_e4a8_dc2b,
            0x1ed3_57e9_faac_c373, 0x124f_83f8_7f21_0508,
            0xbb2a_a991_d746_34ea, 0x7597_c086_6e36_6c07,
            0xf252_5064_ec1e_e5d7, 0xb494_4705_868e_120c,
            0xc14e_c14f_bc86_6719, 0xd74b_1dcd_9524_3ef2,
            0x7841_cfba_e278_8037, 0x2e50_da48_c9f3_e915,
            0x31b5_88a9_4ca0_411e, 0x1477_0add_11fb_29f4,
            0xaefa_cab6_dc18_8683, 0xcdae_0372_ca19_abc5,
            0x473d_e15c_99f7_96f8, 0x3059_ea24_1b8f_f1e2,
            0xf023_ea80_d82b_98b2, 0x739c_2636_e235_dc66,
            0xa80c_1536_d43d_c275, 0x924c_df69_33aa_cd75,
            0xb353_d91b_ddee_5449, 0x2f8b_101d_7e0b_64ec,
            0x2f2f_1dda_96d5_96c9, 0xcf36_c909_bbf1_dd87,
            0x75fe_eaa5_9b68_5a49, 0x81bb_a962_6ad0_b4ad,
            0xb221_9dc1_c91e_b112, 0xd7f8_3ab3_c7f6_6c76,
            0xc5c1_dc62_608f_ea7f, 0x1122_d3d3_57b5_e5a9,
            0x5f15_6386_75bd_26b7, 0xfa02_e2a9_e187_05a2,
            0xa9fd_a284_7bf1_353d, 0xe1e1_00b3_db2f_1a86,
            0x911e_9073_3dd6_d2b2, 0x5ef6_0285_b429_b61a,
            0xe9c2_7db9_3dbe_c46c, 0x594f_46a3_91ed_862a,
            0x3727_bb4d_4d36_c2ba, 0xf859_e484_d4ce_0765,
            0xeb13_d181_2656_9d0e, 0x9c9d_008e_2c86_cdf8,
            0x2abf_a86c_f478_8551, 0x045d_7656_1ab5_8574,
            0x2880_743f_0540_fa1f, 0x1708_3e93_80d9_b7af,
            0x13bb_7015_4e19_59b9, 0x078d_9530_b62a_b487,
        ],
    ],
    // black
    [
        // black pawn
        [
            0x7676_0024_a495_ccba, 0x20bc_5d91_a73f_9faf,
            0x53ce_8501_0419_f93c, 0x0cf3_ab98_e05d_b41c,
            0x0bb3_353c_831e_75db, 0xfe2a_6b2f_cd55_3dc1,
            0xf6d4_e02c_77d0_1d36, 0xe99d_0940_5a87_af9a,
            0x988b_7d8e_af37_5365, 0xbf6e_185e_aa8c_6401,
            0xeb6c_006b_269d_7d9d, 0x0baf_4201_1e96_4860,
            0xcde0_1eca_c8ed_293a, 0x77f6_a313_8b6f_2a4f,
            0x730e_245f_5d05_3dbb, 0xbd69_7d51_6c90_d9cf,
            0x2db2_9ee1_c529_a7a2, 0xdc7c_2325_ce19_4ab5,
            0x43cf_32cf_caee_0450, 0x1cec_47fe_35c9_5d70,
            0xbb30_2fbc_f65b_f943, 0x6d96_6b3d_0a20_fdc7,
            0x383f_8b67_5a22_0b5b, 0x393b_e846_d64f_d43a,
            0x19a2_c344_d216_fcfe, 0xaa5e_5250_50b7_dd18,
            0xf49b_c437_ec4d_f44a, 0xa5c1_26f3_38fe_d682,
            0x7728_60a1_989f_ecac, 0x7829_8ea1_fa07_9ce0,
            0x7e56_0cff_c55e_4187, 0x517d_d67b_7b59_3b8d,
            0xe049_93c6_4378_71c2, 0x2c5a_8b3c_b4b9_adb9,
            0x02ef_3135_a770_966f, 0xaeb7_a120_6609_09c4,
            0xd958_103d_5fd3_68d8, 0xe53c_e101_ccb1_1790,
            0x4b27_71d3_e516_1eac, 0x784c_e71d_4f18_2093,
            0x5aab_e033_ba04_d514, 0x1f00_5c2c_de04_1ea8,
            0x977c_afef_ef28_bdef, 0xa18c_b5cd_3adb_4ce0,
            0x8cae_b8d6_fedf_c478, 0xb1c2_b761_691d_83d2,
            0xe306_a10c_22f9_e915, 0x3db2_20df_5096_b935,
            0xe0bb_4ce2_b810_2617, 0xa5a6_753f_fd18_70bc,
            0xc9a0_0638_de0a_49a8, 0x6be6_8683_f7ca_ea7e,
            0x63d6_b689_4862_d32b, 0x8a0e_8176_ed4d_6169,
            0x8e63_1ee9_1c2d_8ccd, 0x94e0_87aa_3345_ba7a,
            0x6833_518a_ccff_4c10, 0x3cdd_825e_c716_9bd0,
            0xff7f_d09e_251b_fe8b, 0x8710_bd3a_0ed8_00c5,
            0xe293_9e9b_089d_ea23, 0x4ed0_0419_fcc7_069c,
            0x345f_3c49_0e7f_d281, 0xa60e_6f1e_a84a_8579,
        ],
        // black knight
        [
            0x3e69_7351_34cb_db8a, 0x1872_0912_bdaa_8786,
            0x71ef_e62f_7ad1_700c, 0x6322_ba7a_1779_3afa,
            0x64b3_f38b_0001_c8dc, 0x7c13_a7cc_1c5d_ddc6,
            0x9f0c_b4b9_1c28_95ff, 0x4a6c_7747_228a_af64,
            0xd77b_2742_9fce_01ae, 0xd351_5264_29cc_1c66,
            0xdf59_d608_32d0_801f, 0xec88_2707_d304_032f,
            0xc090_9ae2_2284_c770, 0x0652_a3a5_44e5_784e,
            0xe713_b5d0_99a8_fd30, 0x98e1_8cc2_12e0_2e0d,
            0x0e18_2b35_7bb6_9553, 0x3302_c241_2e6a_b520,
            0xee41_56de_045f_5486, 0x80c9_cb71_060c_e698,
            0x573a_e5b4_db2b_6631, 0xe26d_08d1_38d6_e96e,
            0x7482_f44e_82a6_7c9c, 0x549a_baf6_b035_705b,
            0x7a55_e4e3_e983_8010, 0x7ce5_e610_853e_fed2,
            0xdcf6_9b3e_6b93_67cf, 0xa62a_b9e6_d1f6_3e3e,
            0x81d3_31aa_bf97_7657, 0x5a00_8e67_721d_aef2,
            0x0f37_e612_2633_3840, 0x0d8e_b50b_c28a_4a59,
            0x9c8c_f2b6_ad8a_539f, 0x0967_57af_a120_fe5f,
            0x8a65_4235_8459_6bd7, 0x4d3a_5dad_4ae8_4425,
            0x36c0_72da_268a_1c26, 0xb17e_d843_38da_7661,
            0x974c_695a_a49a_516a, 0xf26a_c7a6_c757_f61c,
            0x809e_c77a_664b_19e9, 0xa0d8_224a_4772_9e9c,
            0x8695_f74b_907e_580d, 0x2eea_e831_1475_83ae,
            0xef68_1ff3_a531_a369, 0x6f8b_b454_6b48_77a3,
            0x2813_4532_0beb_097d, 0xe92a_ae7f_0cb3_60a9,
            0x8626_070f_f032_4e02, 0x4d48_f829_2687_f769,
            0xe821_f36f_4ab9_c0b7, 0x3fa9_af17_9b43_22f9,
            0xf2ec_1f74_f96a_bcfb, 0x5957_68ee_82af_c89a,
            0x4010_caea_8186_6db0, 0xfcb6_9af0_a97f_8fd4,
            0xcdb8_241e_7665_7e2b, 0x215d_b982_e911_cdd7,
            0x66b1_deed_86b3_f730, 0xb77e_fb52_6bba_8547,
            0x85e1_35da_e117_a547, 0xa3f9_9420_5544_29d0,
            0x6def_1884_f8a2_0bd4, 0x1881_3157_0627_859c,
        ],
        // black bishop
        [
            0x9f79_eb85_d291_a124, 0xcb1a_bec7_b08f_16d6,
            0x7433_cc4d_4686_cd4b, 0x3a78_945c_9e16_732e,
            0x2f1f_c930_25a8_c65c, 0x7f0b_74d5_5e5d_b3c7,
            0xdac5_1bff_1519_cc0c, 0x0283_7c2b_f5da_3012,
            0x42fc_1493_0cdd_e1b3, 0x7c7d_4454_cda7_d6a9,
            0x1230_c514_908e_f99e, 0x40a6_4496_2a91_0500,
            0x520e_f970_8cf0_8ee2, 0xc117_2b79_e8f7_d38a,
            0xfaaa_b000_0c07_8f01, 0x47bd_78a1_dfb8_2bb2,
            0xc480_39c7_bc9f_7f12, 0xc6b6_4fef_8bbb_6b7c,
            0xe2e1_e0c8_3d6c_da40, 0xe4ea_7ba1_8ce6_50d3,
            0xa5b2_7350_3733_c1c9, 0x1952_d76d_0c7b_64b5,
            0x428c_5b42_9496_38e5, 0xbca7_eab2_5c39_24d6,
            0x2e9a_f8b9_9185_512b, 0x767b_4c4d_fcbd_b860,
            0x83d5_cf90_ef69_de58, 0x4fe9_86ac_c0f5_12f2,
            0xe572_b7a6_5677_82ab, 0x0367_9724_f3a6_0703,
            0x3899_a7e7_cfe7_271a, 0xd65e_cfa7_3b5d_ef42,
            0x4fc8_e47c_4fdb_3d1f, 0x669a_d666_7847_449b,
            0x5875_509b_7938_79eb, 0x5ab7_baf7_fec0_ad09,
            0x73cd_64e9_20d5_152e, 0xd218_9d07_60a5_cd49,
            0x64aa_d363_98a0_f7cd, 0xc60c_2cb6_6eb0_6e1b,
            0xa610_3232_76b0_b12e, 0x4b35_6124_3a6c_53b2,
            0x8edc_1ef3_5f6b_7bf6, 0xc751_1dcb_a87d_7c68,
            0x6075_2c4a_5521_1562, 0xbcf4_23ad_9260_59f8,
            0x3f98_93cc_32d3_ac99, 0xc3e1_9777_8dd2_feac,
            0x1f97_16a0_4306_b996, 0xa1fb_9a16_2f7b_2661,
            0x99b7_af52_56d1_1d99, 0x3e19_ed6a_0599_7943,
            0x836a_b7c3_5c93_892a, 0x5319_630b_96c2_649c,
            0xd797_ee2f_38eb_c627, 0x5f0e_df54_6801_6c23,
            0x303e_54b9_8ff7_2e75, 0x4764_0f23_6ccf_e4a7,
            0x9f1b_3eee_3fea_4a33, 0x904e_8b01_4ac6_9ef6,
            0x2d12_0129_8973_4a97, 0xd2e6_06a3_8a07_2863,
            0xa992_9de2_4594_d8d3, 0x5631_7ecc_7c42_7f94,
        ],
        // black rook
        [
            0x52bf_57f9_22a4_ec59, 0xbc39_7d39_13e3_7aa8,
            0x30e6_9538_d695_b0c1, 0x5c5e_b34f_1f4b_a084,
            0xbf88_459c_6f04_9d43, 0xeb55_9a40_f2c2_4b4f,
            0xe6ba_a4bd_4302_3b9b, 0x2f82_8cd4_daa7_1a4e,
            0x6016_44cb_64d8_a58d, 0x57a8_5f2f_7a98_7bb6,
            0xcc35_f000_7298_632f, 0xfef4_cb1f_9308_acd3,
            0x4a30_8612_4dac_529a, 0xe3b5_65c9_6ce6_30ec,
            0x46aa_b691_ec68_5037, 0xa3cc_dcdf_0511_683c,
            0xad22_e556_0ff2_3d2d, 0x5b37_2995_dc7c_faa6,
            0x837a_3a8f_e80c_6fb5, 0x7247_1fff_4523_dc0d,
            0x3cfa_dc01_9577_8f57, 0x426b_419a_1f4e_ca61,
            0xbf41_917e_0b1b_5d4e, 0x7997_021e_3dc0_47da,
            0x28dc_a2c0_0513_ef94, 0x4db0_bdde_12b5_3dec,
            0xe68d_911b_ac53_735d, 0xa291_7955_daf0_a7e6,
            0x8963_6bb6_048b_a204, 0x3e7c_21a9_3725_de58,
            0x1d32_e13b_7173_b19e, 0x0a26_6e39_078b_5e9f,
            0x741f_de3c_77a9_b0cc, 0x229d_b38a_6859_3444,
            0xf92e_7baa_5ccd_eb7c, 0x34b7_f022_e183_7c00,
            0x2123_3d24_7184_2c9c, 0x32da_1c23_36f4_d297,
            0x7e1b_230a_9f89_6f0f, 0xf737_4cbb_ca1c_e3dc,
            0xcfa4_e07c_4d00_d653, 0xcddc_8cb7_2892_b895,
            0x2642_99c3_c59f_95ae, 0x3324_63d0_3719_d407,
            0x56b0_ebd7_217e_c2cf, 0x4c76_e67d_6351_ac61,
            0x8637_a1e7_1081_ef34, 0xc257_d064_69dc_8b62,
            0x8ef9_6a1f_08c5_c9d9, 0xac3a_9bb5_89ca_6ed7,
            0x580c_0f24_157e_f8de, 0x2fa0_c008_7180_12d5,
            0x02b1_37e3_eb1d_d13f, 0x4d62_4385_cc85_e9d8,
            0xfb1c_73bb_f4d2_90d0, 0x193f_9179_1cdc_6920,
            0x907f_652b_c699_4504, 0x4110_1996_75d7_b9f0,
            0xb564_3171_be76_9743, 0x5a2b_9380_c185_9298,
            0xee7e_f7d7_a69d_4dc6, 0xe4e3_e275_753b_b461,
            0x4374_f791_d993_6e51, 0xb433_c6ca_0026_69d0,
        ],
        // black queen
        [
            0x572f_7d17_fcbe_4dc6, 0x3a60_3776_c092_9634,
            0xc8db_edbf_af20_4a98, 0x83f5_f6e7_83ba_4966,
            0x0cda_77d1_d35b_6ded, 0x0394_ebaf_6ad4_b0b8,
            0x1c94_2f04_1625_ec01, 0x4bba_54c5_bcc2_7d59,
            0xbb45_d586_d59f_1c6a, 0x7c8c_361f_e2cb_6e3f,
            0x20d5_a67c_d2cd_c73d, 0xaf74_a60d_3467_4223,
            0x4cac_ca46_2c04_8208, 0xc012_f0c8_1743_b722,
            0x5a75_9277_fea9_8bc4, 0x4fed_7b37_0e90_973e,
            0xd3d9_c389_b202_9926, 0x57b1_b9fc_ebeb_b73a,
            0x8390_2294_b7ee_0ce2, 0x75de_4666_9354_3ba8,
            0xbf68_35ae_83bf_29d2, 0x7034_16d0_ab19_6e10,
            0x7811_a5b9_928b_e71e, 0x0a52_247b_1fac_ea89,
            0x9ed7_f8b1_a3bb_30bc, 0x6937_550e_9ab1_f438,
            0x9a92_6bd5_aba5_8901, 0xb8ab_5227_ebd9_4d7c,
            0x9675_8d31_9d6d_320f, 0xaccc_15b5_e947_06cb,
            0x6e8f_d463_e494_4049, 0xc729_560f_7c88_557d,
            0xdf19_e2b0_49d6_3937, 0xfc37_a8f1_687a_aa0a,
            0x64eb_42ac_0a4e_2c71, 0x7311_28e3_d399_eb7d,
            0xeeb5_781e_1b02_867c, 0x9ebb_a49f_c97d_f41e,
            0x17cf_f0d7_ee41_37ae, 0xdd67_e489_b5a9_2995,
            0x26be_efa8_9384_2b8e, 0x1f79_5e51_50d3_5104,
            0x72d2_dab4_b5ae_7742, 0xd537_c340_994b_8de3,
            0x255f_6ec2_cee6_17f1, 0x876c_31da_3641_d096,
            0x8484_394f_e469_9440, 0x0c74_77cd_14ef_d806,
            0x168a_6381_4003_5647, 0x406b_bd59_25f9_4fbd,
            0xf63e_4847_e819_1da5, 0x8470_27a9_e088_8243,
            0xa1d7_8833_0a66_7b88, 0x0e4d_7bcc_97ef_8dc5,
            0x6750_4deb_28e7_e4e3, 0x3ee0_903a_bfdc_0746,
            0x523a_3e3a_9b8a_8916, 0xcb1a_6726_f26a_7ed4,
            0x287f_1f14_e49e_c5c8, 0x8079_2002_ea91_6591,
            0x948b_992a_691c_c2ba, 0xd34c_8ae7_acc8_6c86,
            0x14da_97cb_5946_7acb, 0x3c85_d592_1ddf_f351,
        ],
        // black king
        [
            0x3221_0ce1_c6e3_9187, 0xa9e3_b336_f92f_3981,
            0x4f30_b03b_e76f_b5ae, 0x84db_c1ac_042e_5ebe,
            0x2daf_b053_0c1b_c721, 0x2acc_23ce_9959_0c82,
            0xc64d_8c9a_2dfc_3e3a, 0x98eb_a118_fa6b_3ce0,
            0x8349_b2b4_a1ae_ec3f, 0xf847_b195_69d5_a082,
            0xa9eb_1bef_58a1_e03a, 0x5826_e219_9fcc_905c,
            0x044b_ce84_d019_74a8, 0xb7a5_988b_a18d_4f81,
            0xc3a7_0f42_c1b3_6f0d, 0x6087_de1e_413f_d420,
            0xa0c6_34b8_986a_78a5, 0x3633_fd83_3017_5128,
            0xb519_78e4_4476_1db3, 0x2559_b36b_57af_c420,
            0x6499_db05_4783_ef42, 0x51bb_67e0_22b3_dd1d,
            0xf59b_1f20_ba14_593c, 0xf906_6759_7854_b0fd,
            0x2115_3d39_9a0f_3d36, 0x42d8_5a8a_712c_c83b,
            0xfb0f_1e73_e9cc_6863, 0x5f33_4c4f_3774_470d,
            0x1038_5a8a_0626_896e, 0x6620_6535_17bc_1da8,
            0x7ad1_c445_2155_614d, 0x6432_2488_8299_69c5,
            0xfe80_f7fa_ffae_8839, 0xe122_46e2_a2d8_74da,
            0xefa7_cb25_ea73_4663, 0xf6cd_ba9b_3740_0491,
            0xc84d_70b2_4c67_ae18, 0xc034_faaa_91f7_0579,
            0xd1d2_618a_92c2_5b5b, 0x815c_e8c2_faf9_f201,
            0x37d9_9d68_bcb6_fb3a, 0x6a19_c7c8_364f_ba23,
            0x2e34_a41d_7ab5_035d, 0x3f36_47e9_99f3_5a68,
            0x90c3_9bc3_de14_d091, 0x867a_adea_f27c_c31a,
            0x8980_747e_60c6_b952, 0x0b76_ff82_710d_2ad5,
            0xa5b4_18e9_fd16_3605, 0x31f2_20a7_7587_0c60,
            0xf5e6_5751_e532_9750, 0x8983_dd56_1f94_53f3,
            0x37d4_bd06_4949_4bd6, 0xee16_7315_f5d6_5a1b,
            0x1585_0ad7_4eff_82c6, 0x825d_952b_a085_cd11,
            0xf73b_aa32_4671_98d3, 0x0c46_09e7_fc4c_f0b8,
            0x761c_b69c_cd0e_2976, 0x6717_bab1_3ca3_5727,
            0xdf17_6089_c030_1096, 0x4fdd_ed3f_b929_49be,
            0xb796_e444_1bc2_1664, 0xac73_75fe_2095_265e,
        ],
    ],
];

const EP_SQUARE: [u64; File::COUNT] = [
        0x98c6_7268_3e8d_e0b5, 0x7a0d_721a_0d9b_c1d9,
        0x349d_9203_f41f_53f7, 0xb5ab_2c11_937e_82c9,
        0xe0d2_c0c2_b4ba_fd4d, 0xae71_7b95_2502_165c,
        0x9e9b_fd4e_d2b8_1683, 0xc420_8595_06f9_3dab,
];

const CASTLE_FLAGS: [[u64; 4]; Color::COUNT] = [
    [
        0,
        0x31ae_78f7_f1e2_a2bd,
        0x865f_1105_6aa1_8ce6,
        0x31ae_78f7_f1e2_a2bd ^ 0x865f_1105_6aa1_8ce6,
    ],
    [
        0,
        0x9acd_bc5f_97b3_1d77,
        0xfb46_f85b_f7df_8071,
        0x9acd_bc5f_97b3_1d77 ^ 0xfb46_f85b_f7df_8071,
    ],
];

const BLACK_MOVE: u64 = 0xa585_29f9_b891_ace8;
