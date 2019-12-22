//! Counts the number of variations from a given starting position to a specified depth. Defaults
//! to the standard starting position.
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! # Examples
//! You can check the number of variations to a depth of 5 with the following
//! command.
//!
//! ```sh
//! count --depth 5
//! ```
//!
//! This will produce the following output.
//!
//! ```text
//! rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
//!         a4            217832    rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1
//!         b4            216145    rnbqkbnr/pppppppp/8/8/1P6/8/P1PPPPPP/RNBQKBNR b KQkq b3 0 1
//!         c4            240082    rnbqkbnr/pppppppp/8/8/2P5/8/PP1PPPPP/RNBQKBNR b KQkq c3 0 1
//!         d4            361790    rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1
//!         e4            405385    rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
//!         f4            198473    rnbqkbnr/pppppppp/8/8/5P2/8/PPPPP1PP/RNBQKBNR b KQkq f3 0 1
//!         g4            214048    rnbqkbnr/pppppppp/8/8/6P1/8/PPPPPP1P/RNBQKBNR b KQkq g3 0 1
//!         h4            218829    rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR b KQkq h3 0 1
//!         a3            181046    rnbqkbnr/pppppppp/8/8/8/P7/1PPPPPPP/RNBQKBNR b KQkq - 0 1
//!         b3            215255    rnbqkbnr/pppppppp/8/8/8/1P6/P1PPPPPP/RNBQKBNR b KQkq - 0 1
//!         c3            222861    rnbqkbnr/pppppppp/8/8/8/2P5/PP1PPPPP/RNBQKBNR b KQkq - 0 1
//!         d3            328511    rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1
//!         e3            402988    rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1
//!         f3            178889    rnbqkbnr/pppppppp/8/8/8/5P2/PPPPP1PP/RNBQKBNR b KQkq - 0 1
//!         g3            217210    rnbqkbnr/pppppppp/8/8/8/6P1/PPPPPP1P/RNBQKBNR b KQkq - 0 1
//!         h3            181044    rnbqkbnr/pppppppp/8/8/8/7P/PPPPPPP1/RNBQKBNR b KQkq - 0 1
//!         Na3           198572    rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 1 1
//!         Nc3           234656    rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b KQkq - 1 1
//!         Nf3           233491    rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1
//!         Nh3           198502    rnbqkbnr/pppppppp/8/8/8/7N/PPPPPPPP/RNBQKB1R b KQkq - 1 1
//! Depth 5 total:       4865609
//! ```
//!
//! The first line contains the FEN representation of the position being searched. Subsequent lines
//! list all the possible moves from the standard starting position, allong with each move's
//! contribution to the total number of variations, and the FEN representation of the position
//! resulting from each move. The final line gives the total number of variations.
//!
//! To search a different position, we simply pass the FEN representation fo the position to be
//! searched, as in the following example.
//!
//! ```sh
//! count --depth 5 '7K/7p/7k/8/8/8/8/8 w - - 0 1'
//! ```
//!
//! Which produces the following output.
//!
//! ```text
//! 7K/7p/7k/8/8/8/8/8 w - - 0 1
//!         Kg8              342    6K1/7p/7k/8/8/8/8/8 b - - 1 1
//! Depth 5 total:           342
//! ```
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
extern crate clap;
use clap::{Arg, App, crate_version};
use tinman::chess::variations;

fn main() {
    let matches =
        App::new("tinman counts")
            .version(crate_version!())
            .author("Mike Leany")
            .about("Counts the number of variations from a given starting position \
                    to a specified\ndepth. Defaults to the standard starting position.")
            .arg(Arg::with_name("file")
                .short("f")
                .value_name("EPD_FILE")
                .takes_value(true)
                .conflicts_with("depth")
                .conflicts_with("fen")
                .hidden(true)
                .help("An EPD file of positions to search"))
            .arg(Arg::with_name("fen")
                .value_name("FEN_STRING")
                .default_value("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .hide_default_value(true)
                .multiple(true)
                .help("Position to search in Forsyth-Edwards Notation (FEN)"))
            .arg(Arg::with_name("depth")
                .long("depth")
                .short("d")
                .value_name("DEPTH")
                .takes_value(true)
                .required(true)
                .conflicts_with("file")
                .help("Depth to search the position"))
            .get_matches();

    if let Some(file) = matches.value_of("file") {
        read_epd(file);
    } else {
        let depth = matches.value_of("depth").expect("INFALLIBLE").parse().unwrap();

        for fen in matches.values_of("fen").expect("INFALLIBLE") {
            match fen.parse() {
                Ok(pos) => {
                    println!("\n{}", fen);
                    let count = variations::print(&pos, depth);
                    println!("Depth {} total:\t{:12}", depth, count);
                },
                Err(error) => {
                    eprintln!("error: {}", error);
                    return;
                },
            }
        }
    }
}

fn read_epd(file: &str) {
    let epd = match File::open(file) {
        Ok(file) => BufReader::new(file),
        Err(error) => {
            eprintln!("{}: {}", file, error);
            return;
        },
    };

    for (line_num, line) in epd.lines().enumerate() {
        match line {
            Ok(line) => {
                match split_epd_line(&line, line_num) {
                    Ok(_) => {},
                    Err(error) => {
                        eprintln!("{}: line {}: {}", file, line_num, error);
                        return;
                    }
                }
            },
            Err(error) => {
                eprintln!("{}: line {}: {}", file, line_num, error);
                return;
            },
        }
    }
}

fn split_epd_line(line: &str, line_num: usize) -> Result<(), String> {
    let mut fields = line.split(';');
    let fen = match fields.next() {
        Some(fen) => fen,
        None => return Ok(()),
    };
    println!("\nLine {:3}:\t{}", line_num, fen);
    for field in fields {
        use std::cmp::Ordering;
        let nums = field.trim_start_matches('D');
        let nums: Vec<&str> = nums.split_whitespace().collect();
        match nums.len().cmp(&2) {
            Ordering::Less => return Err(format!("\"{}\": not enough fields", field)),
            Ordering::Greater => return Err(format!("\"{}\": too many fields", field)),
            _ => {},
        }

        let depth: usize = match nums[0].parse() {
            Ok(depth) => depth,
            Err(error) => {
                return Err(format!("\"{}\": {}", nums[0], error));
            },
        };
        let expected: usize = match nums[1].parse() {
            Ok(expected) => expected,
            Err(error) => {
                return Err(format!("\"{}\": {}", nums[1], error));
            },
        };

        match fen.parse() {
            Ok(pos) => {
                println!("Depth {} expected:\t{:12}", depth, expected);
                let count = variations::count(&pos, depth);
                println!("Depth {} result:  \t{:12}", depth, count);
                if count != expected {
                    return Err(format!("depth {}: expected {} but counted {}", depth, expected, count));
                }
            },
            Err(error) => return Err(format!("error: {}", error)),
        }
    }

    Ok(())
}
