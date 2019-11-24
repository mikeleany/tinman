//! Counts the number of variations from a given starting position to a specified depth. Defaults
//! to the standard starting position.
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
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
        let nums = field.trim_start_matches('D');
        let nums: Vec<&str> = nums.split_whitespace().collect();
        if nums.len() < 2 {
            return Err(format!("\"{}\": not enough fields", field));
        } else if nums.len() > 2 {
            return Err(format!("\"{}\": too many fields", field));
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
