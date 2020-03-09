//! The tinman chess engine.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

use std::fs::File;
use std::path::PathBuf;
use clap::{App, Arg, SubCommand, crate_version};
use simplelog::{WriteLogger, LevelFilter, Config};
use tinman::chess::variations;
use tinman::engine::Engine;
use tinman::protocol::xboard::Xboard;

fn main() -> Result<(), Error> {
    let _app_dir = dirs::home_dir()
        .map(|home| { home.join(".tinman") })
        .unwrap_or_else(|| PathBuf::from("."));

    let matches =
        App::new("Tinman")
            .version(crate_version!())
            .author("Mike Leany")
            .arg(Arg::with_name("xboard")
                .long("xboard")
                .hidden(true)
                .help("Uses the xboard interface"))
            .arg(Arg::with_name("log")
                .long("log")
                .short("l")
                .global(true)
                .help("Turns on logging"))
            .arg(Arg::with_name("log-file")
                .long("log-file")
                .global(true)
                .value_name("LOG_FILE")
                .takes_value(true)
                .default_value("tinman.log")
                .help("Sets the log file if logging is turned on"))
            .arg(Arg::with_name("log-level")
                .long("log-level")
                .global(true)
                .value_name("LEVEL")
                .takes_value(true)
                .default_value("info")
                .help("Sets the log level if logging is turned on"))
            .subcommand(SubCommand::with_name("counts")
                .about("Counts the number of variations from a given starting position \
                        to a specified\ndepth. Defaults to the standard starting position.")
                .arg(Arg::with_name("depth")
                    .long("depth")
                    .short("d")
                    .value_name("DEPTH")
                    .takes_value(true)
                    .required(true)
                    .help("Depth to search the position"))
                .arg(Arg::with_name("fen")
                    .value_name("FEN_STRING")
                    .default_value("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .hide_default_value(true)
                    .multiple(true)
                    .help("Position to search in Forsyth-Edwards Notation (FEN)")))
            .get_matches();

    let log_file = PathBuf::from(matches.value_of_os("log-file").expect("INFALLIBLE"));
    let log_level = match matches.value_of("log-level") {
        Some("off") => LevelFilter::Off,
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        Some(level) => return Err(Error(format!("{}: invalid log level", level))),
        None => unreachable!(),
    };

    let _logger = if matches.is_present("log") {
        WriteLogger::init(
            log_level,
            Config::default(),
            File::create(&log_file).map_err(|err| {
                Error(format!("{}: {}", log_file.display(), err))
            })?)
    } else {
        WriteLogger::init(LevelFilter::Off, Config::default(), std::io::sink())
    };

    match matches.subcommand() {
        (_, None) => Engine::new(Xboard::new()).run(),
        ("counts", Some(matches)) => {
            let depth = matches
                .value_of("depth")
                .expect("INFALLIBLE")
                .parse()
                .map_err(|_| {Error("depth must be numeric".to_owned())})?;

            println!();
            for fen in matches.values_of("fen").expect("INFALLIBLE") {
                let pos = fen.parse().map_err(|err| {Error(format!("{}: {}", fen, err))})?;
                println!("{}", fen);
                let count = variations::print(&pos, depth);
                println!("Depth {} total:\t{:12}\n", depth, count);
            }
        },
        _ => unreachable!(),
    }


    Ok(())
}

struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for Error { }
