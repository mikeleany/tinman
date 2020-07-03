//! Tests the move generator (chess module)
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////

mod move_gen {
    use chess::variations;

    #[test]
    fn position_001() {
        assert_eq!(
            count("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6),
            119060324
        );
    }

    mod position_002 {
        use super::count;

        #[test]
        fn depth_5() {
            assert_eq!(
                count("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5),
                193690690
            );
        }

        #[test]
        #[ignore]
        fn depth_6() {
            assert_eq!(
                count("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 6),
                8031647685
            );
        }
    }

    #[test]
    fn position_003() { assert_eq!(count("4k3/8/8/8/8/8/8/4K2R w K - 0 1", 6), 764643); }

    #[test]
    fn position_004() { assert_eq!(count("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1", 6), 846648); }

    #[test]
    fn position_005() { assert_eq!(count("4k2r/8/8/8/8/8/8/4K3 w k - 0 1", 6), 899442); }

    #[test]
    fn position_006() { assert_eq!(count("r3k3/8/8/8/8/8/8/4K3 w q - 0 1", 6), 1001523); }

    #[test]
    fn position_007() { assert_eq!(count("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1", 6), 2788982); }

    #[test]
    fn position_008() { assert_eq!(count("r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1", 6), 3517770); }

    #[test]
    fn position_009() { assert_eq!(count("8/8/8/8/8/8/6k1/4K2R w K - 0 1", 6), 185867); }

    #[test]
    fn position_010() { assert_eq!(count("8/8/8/8/8/8/1k6/R3K3 w Q - 0 1", 6), 413018); }

    #[test]
    fn position_011() { assert_eq!(count("4k2r/6K1/8/8/8/8/8/8 w k - 0 1", 6), 179869); }

    #[test]
    fn position_012() { assert_eq!(count("r3k3/1K6/8/8/8/8/8/8 w q - 0 1", 6), 367724); }

    #[test]
    fn position_013() { assert_eq!(count("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 6), 179862938); }

    #[test]
    fn position_014() { assert_eq!(count("r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1", 6), 195629489); }

    #[test]
    fn position_015() { assert_eq!(count("r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1", 6), 184411439); }

    #[test]
    fn position_016() { assert_eq!(count("r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1", 6), 189224276); }

    #[test]
    fn position_017() { assert_eq!(count("1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1", 6), 198328929); }

    #[test]
    fn position_018() { assert_eq!(count("2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1", 6), 185959088); }

    #[test]
    fn position_019() { assert_eq!(count("r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1", 6), 190755813); }

    #[test]
    fn position_020() { assert_eq!(count("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6), 899442); }

    #[test]
    fn position_021() { assert_eq!(count("4k3/8/8/8/8/8/8/R3K3 b Q - 0 1", 6), 1001523); }

    #[test]
    fn position_022() { assert_eq!(count("4k2r/8/8/8/8/8/8/4K3 b k - 0 1", 6), 764643); }

    #[test]
    fn position_023() { assert_eq!(count("r3k3/8/8/8/8/8/8/4K3 b q - 0 1", 6), 846648); }

    #[test]
    fn position_024() { assert_eq!(count("4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1", 6), 3517770); }

    #[test]
    fn position_025() { assert_eq!(count("r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1", 6), 2788982); }

    #[test]
    fn position_026() { assert_eq!(count("8/8/8/8/8/8/6k1/4K2R b K - 0 1", 6), 179869); }

    #[test]
    fn position_027() { assert_eq!(count("8/8/8/8/8/8/1k6/R3K3 b Q - 0 1", 6), 367724); }

    #[test]
    fn position_028() { assert_eq!(count("4k2r/6K1/8/8/8/8/8/8 b k - 0 1", 6), 185867); }

    #[test]
    fn position_029() { assert_eq!(count("r3k3/1K6/8/8/8/8/8/8 b q - 0 1", 6), 413018); }

    #[test]
    fn position_030() { assert_eq!(count("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", 6), 179862938); }

    #[test]
    fn position_031() { assert_eq!(count("r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1", 6), 198328929); }

    #[test]
    fn position_032() { assert_eq!(count("r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1", 6), 185959088); }

    #[test]
    fn position_033() { assert_eq!(count("r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1", 6), 190755813); }

    #[test]
    fn position_034() { assert_eq!(count("1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1", 6), 195629489); }

    #[test]
    fn position_035() { assert_eq!(count("2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1", 6), 184411439); }

    #[test]
    fn position_036() { assert_eq!(count("r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1", 6), 189224276); }

    #[test]
    fn position_037() { assert_eq!(count("8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1", 6), 8107539); }

    #[test]
    fn position_038() { assert_eq!(count("8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1", 6), 2594412); }

    #[test]
    fn position_039() { assert_eq!(count("8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1", 6), 19870403); }

    #[test]
    fn position_040() { assert_eq!(count("K7/8/2n5/1n6/8/8/8/k6N w - - 0 1", 6), 588695); }

    #[test]
    fn position_041() { assert_eq!(count("k7/8/2N5/1N6/8/8/8/K6n w - - 0 1", 6), 688780); }

    #[test]
    fn position_042() { assert_eq!(count("8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1", 6), 8503277); }

    #[test]
    fn position_043() { assert_eq!(count("8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1", 6), 3147566); }

    #[test]
    fn position_044() { assert_eq!(count("8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1", 6), 4405103); }

    #[test]
    fn position_045() { assert_eq!(count("K7/8/2n5/1n6/8/8/8/k6N b - - 0 1", 6), 688780); }

    #[test]
    fn position_046() { assert_eq!(count("k7/8/2N5/1N6/8/8/8/K6n b - - 0 1", 6), 588695); }

    #[test]
    fn position_047() { assert_eq!(count("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6), 22823890); }

    #[test]
    fn position_048() { assert_eq!(count("8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1", 6), 28861171); }

    #[test]
    fn position_049() { assert_eq!(count("k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1", 6), 7881673); }

    #[test]
    fn position_050() { assert_eq!(count("K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1", 6), 7382896); }

    #[test]
    fn position_051() { assert_eq!(count("B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1", 6), 9250746); }

    #[test]
    fn position_052() { assert_eq!(count("8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1", 6), 29027891); }

    #[test]
    fn position_053() { assert_eq!(count("k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1", 6), 7382896); }

    #[test]
    fn position_054() { assert_eq!(count("K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1", 6), 7881673); }

    #[test]
    fn position_055() { assert_eq!(count("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6), 44956585); }

    #[test]
    fn position_056() { assert_eq!(count("R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1", 6), 525169084); }

    #[test]
    fn position_057() { assert_eq!(count("7k/RR6/8/8/8/8/rr6/7K b - - 0 1", 6), 44956585); }

    #[test]
    fn position_058() { assert_eq!(count("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6), 524966748); }

    #[test]
    fn position_059() { assert_eq!(count("6kq/8/8/8/8/8/8/7K w - - 0 1", 6), 391507); }

    #[test]
    fn position_060() { assert_eq!(count("6KQ/8/8/8/8/8/8/7k b - - 0 1", 6), 391507); }

    #[test]
    fn position_061() { assert_eq!(count("K7/8/8/3Q4/4q3/8/8/7k w - - 0 1", 6), 3370175); }

    #[test]
    fn position_062() { assert_eq!(count("6qk/8/8/8/8/8/8/7K b - - 0 1", 6), 419369); }

    #[test]
    fn position_063() { assert_eq!(count("6KQ/8/8/8/8/8/8/7k b - - 0 1", 6), 391507); }

    #[test]
    fn position_064() { assert_eq!(count("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6), 3370175); }

    #[test]
    fn position_065() { assert_eq!(count("8/8/8/8/8/K7/P7/k7 w - - 0 1", 6), 6249); }

    #[test]
    fn position_066() { assert_eq!(count("8/8/8/8/8/7K/7P/7k w - - 0 1", 6), 6249); }

    #[test]
    fn position_067() { assert_eq!(count("K7/p7/k7/8/8/8/8/8 w - - 0 1", 6), 2343); }

    #[test]
    fn position_068() { assert_eq!(count("7K/7p/7k/8/8/8/8/8 w - - 0 1", 6), 2343); }

    #[test]
    fn position_069() { assert_eq!(count("8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1", 6), 34834); }

    #[test]
    fn position_070() { assert_eq!(count("8/8/8/8/8/K7/P7/k7 b - - 0 1", 6), 2343); }

    #[test]
    fn position_071() { assert_eq!(count("8/8/8/8/8/7K/7P/7k b - - 0 1", 6), 2343); }

    #[test]
    fn position_072() { assert_eq!(count("K7/p7/k7/8/8/8/8/8 b - - 0 1", 6), 6249); }

    #[test]
    fn position_073() { assert_eq!(count("7K/7p/7k/8/8/8/8/8 b - - 0 1", 6), 6249); }

    #[test]
    fn position_074() { assert_eq!(count("8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1", 6), 34822); }

    #[test]
    fn position_075() { assert_eq!(count("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1", 6), 11848); }

    #[test]
    fn position_076() { assert_eq!(count("4k3/4p3/4K3/8/8/8/8/8 b - - 0 1", 6), 11848); }

    #[test]
    fn position_077() { assert_eq!(count("8/8/7k/7p/7P/7K/8/8 w - - 0 1", 6), 10724); }

    #[test]
    fn position_078() { assert_eq!(count("8/8/k7/p7/P7/K7/8/8 w - - 0 1", 6), 10724); }

    #[test]
    fn position_079() { assert_eq!(count("8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1", 6), 53138); }

    #[test]
    fn position_080() { assert_eq!(count("8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1", 6), 157093); }

    #[test]
    fn position_081() { assert_eq!(count("8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1", 6), 158065); }

    #[test]
    fn position_082() { assert_eq!(count("k7/8/3p4/8/3P4/8/8/7K w - - 0 1", 6), 20960); }

    #[test]
    fn position_083() { assert_eq!(count("8/8/7k/7p/7P/7K/8/8 b - - 0 1", 6), 10724); }

    #[test]
    fn position_084() { assert_eq!(count("8/8/k7/p7/P7/K7/8/8 b - - 0 1", 6), 10724); }

    #[test]
    fn position_085() { assert_eq!(count("8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1", 6), 53138); }

    #[test]
    fn position_086() { assert_eq!(count("8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1", 6), 158065); }

    #[test]
    fn position_087() { assert_eq!(count("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6), 157093); }

    #[test]
    fn position_088() { assert_eq!(count("k7/8/3p4/8/3P4/8/8/7K b - - 0 1", 6), 21104); }

    #[test]
    fn position_089() { assert_eq!(count("7k/3p4/8/8/3P4/8/8/K7 w - - 0 1", 6), 32191); }

    #[test]
    fn position_090() { assert_eq!(count("7k/8/8/3p4/8/8/3P4/K7 w - - 0 1", 6), 30980); }

    #[test]
    fn position_091() { assert_eq!(count("k7/8/8/7p/6P1/8/8/K7 w - - 0 1", 6), 41874); }

    #[test]
    fn position_092() { assert_eq!(count("k7/8/7p/8/8/6P1/8/K7 w - - 0 1", 6), 29679); }

    #[test]
    fn position_093() { assert_eq!(count("k7/8/8/6p1/7P/8/8/K7 w - - 0 1", 6), 41874); }

    #[test]
    fn position_094() { assert_eq!(count("k7/8/6p1/8/8/7P/8/K7 w - - 0 1", 6), 29679); }

    #[test]
    fn position_095() { assert_eq!(count("k7/8/8/3p4/4p3/8/8/7K w - - 0 1", 6), 22886); }

    #[test]
    fn position_096() { assert_eq!(count("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6), 28662); }

    #[test]
    fn position_097() { assert_eq!(count("7k/3p4/8/8/3P4/8/8/K7 b - - 0 1", 6), 32167); }

    #[test]
    fn position_098() { assert_eq!(count("7k/8/8/3p4/8/8/3P4/K7 b - - 0 1", 6), 30749); }

    #[test]
    fn position_099() { assert_eq!(count("k7/8/8/7p/6P1/8/8/K7 b - - 0 1", 6), 41874); }

    #[test]
    fn position_100() { assert_eq!(count("k7/8/7p/8/8/6P1/8/K7 b - - 0 1", 6), 29679); }

    #[test]
    fn position_101() { assert_eq!(count("k7/8/8/6p1/7P/8/8/K7 b - - 0 1", 6), 41874); }

    #[test]
    fn position_102() { assert_eq!(count("k7/8/6p1/8/8/7P/8/K7 b - - 0 1", 6), 29679); }

    #[test]
    fn position_103() { assert_eq!(count("k7/8/8/3p4/4p3/8/8/7K b - - 0 1", 6), 22579); }

    #[test]
    fn position_104() { assert_eq!(count("k7/8/3p4/8/8/4P3/8/7K b - - 0 1", 6), 28662); }

    #[test]
    fn position_105() { assert_eq!(count("7k/8/8/p7/1P6/8/8/7K w - - 0 1", 6), 41874); }

    #[test]
    fn position_106() { assert_eq!(count("7k/8/p7/8/8/1P6/8/7K w - - 0 1", 6), 29679); }

    #[test]
    fn position_107() { assert_eq!(count("7k/8/8/1p6/P7/8/8/7K w - - 0 1", 6), 41874); }

    #[test]
    fn position_108() { assert_eq!(count("7k/8/1p6/8/8/P7/8/7K w - - 0 1", 6), 29679); }

    #[test]
    fn position_109() { assert_eq!(count("k7/7p/8/8/8/8/6P1/K7 w - - 0 1", 6), 55338); }

    #[test]
    fn position_110() { assert_eq!(count("k7/6p1/8/8/8/8/7P/K7 w - - 0 1", 6), 55338); }

    #[test]
    fn position_111() { assert_eq!(count("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6), 199002); }

    #[test]
    fn position_112() { assert_eq!(count("7k/8/8/p7/1P6/8/8/7K b - - 0 1", 6), 41874); }

    #[test]
    fn position_113() { assert_eq!(count("7k/8/p7/8/8/1P6/8/7K b - - 0 1", 6), 29679); }

    #[test]
    fn position_114() { assert_eq!(count("7k/8/8/1p6/P7/8/8/7K b - - 0 1", 6), 41874); }

    #[test]
    fn position_115() { assert_eq!(count("7k/8/1p6/8/8/P7/8/7K b - - 0 1", 6), 29679); }

    #[test]
    fn position_116() { assert_eq!(count("k7/7p/8/8/8/8/6P1/K7 b - - 0 1", 6), 55338); }

    #[test]
    fn position_117() { assert_eq!(count("k7/6p1/8/8/8/8/7P/K7 b - - 0 1", 6), 55338); }

    #[test]
    fn position_118() { assert_eq!(count("3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1", 6), 199002); }

    #[test]
    fn position_119() { assert_eq!(count("8/Pk6/8/8/8/8/6Kp/8 w - - 0 1", 6), 1030499); }

    #[test]
    fn position_120() { assert_eq!(count("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6), 37665329); }

    #[test]
    fn position_121() { assert_eq!(count("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6), 28859283); }

    #[test]
    fn position_122() { assert_eq!(count("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6), 71179139); }

    #[test]
    fn position_123() { assert_eq!(count("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6), 1030499); }

    #[test]
    fn position_124() { assert_eq!(count("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6), 37665329); }

    #[test]
    fn position_125() { assert_eq!(count("8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1", 6), 28859283); }

    #[test]
    fn position_126() { assert_eq!(count("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6), 71179139); }

    fn count(fen: &str, depth: usize) -> usize {
        println!("\n{}", fen);
        let pos = fen.parse().unwrap();

        let count = variations::print(&pos, depth);
        println!("Depth {} total:\t{:12}", depth, count);

        count
    }
}
