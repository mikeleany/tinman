//! Module for counting and printing the number of variations from a given position
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::chess::*;

/// Print the number of variations of the given `depth` for each legal move from `pos`
pub fn print(pos: &Position, depth: usize) -> usize {
    if depth < 1 {
        return 1;
    }

    let mut total = 0;

    let moves = pos.moves();
    for m in moves {
        if let Ok(pos) = m.make() {
            let count = count(&pos, depth - 1);
            total += count;
            println!("\t{:7}\t{:12}\t{}", m, count, pos);
        }
    }

    total
}

/// Count the number of variations of the given `depth` from `pos`
pub fn count(pos: &Position, depth: usize) -> usize {
    if depth < 1 {
        return 1;
    }

    let mut total = 0;

    let moves = pos.moves();
    for m in moves {
        if let Ok(pos) = m.make() {
            total += count(&pos, depth - 1);
        }
    }

    total
}
