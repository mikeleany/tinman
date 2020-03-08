//! A utility for automatically testing chess engines.
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

use std::path::{Path, PathBuf};
use std::fs::{read_to_string, write, create_dir, read_dir, File, OpenOptions};
use std::io::Write;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::time::Duration;
use clap::{App, Arg, SubCommand, AppSettings, ArgMatches, crate_version};
use log::error;
use simplelog::{WriteLogger, LevelFilter, Config};
use rand::seq::SliceRandom;
use chrono::Local;
use tinman::protocol::xboard::XboardClient;
use tinman::client::GameSetup;
use tinman::chess::game::{MoveSequence, TimeControl};
use tinman::pgn::read_pgn_games;

fn main() -> Result<(), Error> {
    let matches =
        App::new("Chess Test")
            .version(crate_version!())
            .author("Mike Leany")
            .about("Tests one or more chess engines against a number of opponents.")
            .setting(AppSettings::SubcommandRequired)
            .arg(Arg::with_name("dir")
                .long("dir")
                .short("d")
                .global(true)
                .takes_value(true)
                .value_name("DIRECTORY")
                .help("The root of the testing directory structure"))
            .subcommand(SubCommand::with_name("init")
                .about("Sets up a new test environment in the current or given directory"))
            .subcommand(SubCommand::with_name("run")
                .about("Runs the tests")
                .arg(Arg::with_name("log")
                    .long("log")
                    .short("l")
                    .help("Turns on logging"))
                .arg(Arg::with_name("log-file")
                    .long("log-file")
                    .value_name("LOG_FILE")
                    .takes_value(true)
                    .default_value("testing.log")
                    .help("Sets the log file if logging is turned on"))
                .arg(Arg::with_name("log-level")
                    .long("log-level")
                    .value_name("LEVEL")
                    .takes_value(true)
                    .default_value("info")
                    .help("Sets the log level if logging is turned on")))
            .subcommand(SubCommand::with_name("add")
                .about("Adds a new engine")
                .arg(Arg::with_name("candidate")
                    .long("candidate")
                    .help("Adds a new candidate engine (default)"))
                .arg(Arg::with_name("opponent")
                    .long("opponent")
                    .conflicts_with("candidate")
                    .help("Add a new opponent engine"))
                .arg(Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .takes_value(true)
                    .value_name("EXECUTABLE")
                    .help("A name for the engine (defaults to the executable name)"))
                .arg(Arg::with_name("bin")
                    .takes_value(true)
                    .value_name("EXECUTABLE")
                    .required(true)
                    .help("The engine's executable"))
                .arg(Arg::with_name("args")
                    .takes_value(true)
                    .value_name("ARGUMENTS")
                    .multiple(true)
                    .help("Command line arguments to pass to the engine")))
            .subcommand(SubCommand::with_name("remove")
                .visible_alias("rm")
                .about("Remove an engine")
                .arg(Arg::with_name("candidate")
                    .long("candidate")
                    .help("Removes a candidate engine (default)"))
                .arg(Arg::with_name("opponent")
                    .long("opponent")
                    .conflicts_with("candidate")
                    .help("Removes an opponent engine"))
                .arg(Arg::with_name("name")
                    .takes_value(true)
                    .required(true)
                    .value_name("ENGINE_NAME")
                    .help("The name of the engine to remove")))
            .subcommand(SubCommand::with_name("openings")
                .about("Adds new openings")
                .arg(Arg::with_name("files")
                    .takes_value(true)
                    .value_name("FILES")
                    .required(true)
                    .multiple(true)
                    .help("PGN or move sequence files containing opening sequences")))
            .get_matches();

    let dir = PathBuf::from(matches.value_of("dir").unwrap_or(""));
    let paths = Paths {
        candidates_file: dir.join("candidates.yaml"),
        opponents_file: dir.join("opponents.yaml"),
        opening_file: dir.join("openings.yaml"),
        bin_dir: dir.join("bin"),
        games_dir: dir.join("games"),
        dir,
    };

    match matches.subcommand() {
        ("init", Some(_matches)) => {
            if !paths.dir.as_os_str().is_empty() && !paths.dir.is_dir() {
                println!("Creating '{}'.", paths.dir.display());
                create_dir(paths.dir)?;
            } else if paths.candidates_file.exists() {
                return Err(Error(format!("'{}' already exists", paths.candidates_file.display())));
            } else if paths.opponents_file.exists() {
                return Err(Error(format!("'{}' already exists", paths.opponents_file.display())));
            } else if paths.opening_file.exists() {
                return Err(Error(format!("'{}' already exists", paths.opening_file.display())));
            }

            if !paths.bin_dir.is_dir() {
                println!("Creating '{}'.", paths.bin_dir.display());
                create_dir(paths.bin_dir)?;
            }
            if !paths.games_dir.is_dir() {
                println!("Creating '{}'.", paths.games_dir.display());
                create_dir(paths.games_dir)?;
            }

            let engines: HashMap<String, Vec<String>> = HashMap::new();
            println!("Creating '{}'.", paths.candidates_file.display());
            write_engine_file(&paths.candidates_file, &engines)?;
            println!("Creating '{}'.", paths.opponents_file.display());
            write_engine_file(&paths.opponents_file, &engines)?;
            let openings: HashMap<String, String> = HashMap::new();
            println!("Creating '{}'.", paths.opening_file.display());
            write_opening_file(&paths.opening_file, &openings)?;
        },
        ("add", Some(matches)) => {
            let bin = matches.value_of("bin").expect("INFALLIBLE").to_string();
            let mut cmd = vec![ bin ];
            if let Some(args) = matches.values_of("args") {
                cmd.extend(args.map(|s| { s.to_string() }));
            }

            let name = matches.value_of("name")
                .unwrap_or(
                    &PathBuf::from(&cmd[0])
                    .file_name()
                    .expect("INFALLIBLE")
                    .to_string_lossy())
                .to_string();

            let file = if matches.is_present("opponent") { paths.opponents_file } else { paths.candidates_file };
            let mut engines = read_engine_file(&file)?;
            engines.insert(name, cmd);
            write_engine_file(&file, &engines)?;
        },
        ("remove", Some(matches)) => {
            let name = matches.value_of("name").expect("INFALLIBLE");
            let file = if matches.is_present("opponent") { paths.opponents_file } else { paths.candidates_file };
            let mut engines = read_engine_file(&file)?;
            engines.remove(name);
            write_engine_file(&file, &engines)?;
        },
        ("openings", Some(matches)) => {
            let mut openings = read_opening_file(&paths.opening_file)?;
            for file in matches.values_of("files").expect("INFALLIBLE") {
                for opening in read_pgn_openings(&PathBuf::from(file))? {
                    let opening = opening.parse::<MoveSequence>()?;
                    let final_pos = opening.final_position().to_string();
                    match openings.entry(final_pos) {
                        Entry::Vacant(entry) => {
                            entry.insert(opening.to_string());
                        },
                        Entry::Occupied(entry) => {
                            println!("duplicate opening position ignored:\n\t{}", entry.key());
                        },
                    }
                }
            }

            write_opening_file(&paths.opening_file, &openings)?;
        },
        ("run", Some(matches)) => {
            run(matches, &paths)?;
        },
        _ => unreachable!(),
    }

    Ok(())
}

struct Paths {
    dir: PathBuf,
    candidates_file: PathBuf,
    opponents_file: PathBuf,
    opening_file: PathBuf,
    bin_dir: PathBuf,
    games_dir: PathBuf,
}

fn run(matches: &ArgMatches, paths: &Paths) -> Result<(), Error> {
    let log_file = paths.dir.join(matches.value_of_os("log-file").expect("INFALLIBLE"));
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

    let candidates: HashMap<String, Vec<String>> = read_engine_file(&paths.candidates_file)?;
    let opponents: HashMap<String, Vec<String>> = read_engine_file(&paths.opponents_file)?;
    let openings = read_opening_file(&paths.opening_file)?;

    // create set of games to be played
    let mut white_games = HashSet::new();
    let mut black_games = HashSet::new();
    for eng in candidates.keys() {
        for opp in opponents.keys() {
            for opening in openings.keys() {
                white_games.insert((eng.to_owned(), opp.to_owned(), opening.to_owned()));
                black_games.insert((eng.to_owned(), opp.to_owned(), opening.to_owned()));
            }
        }
    }

    // remove games that have already been played
    for entry in read_dir(&paths.games_dir)? {
        let pgn_file = entry?.path();
        println!("reading {}\n", pgn_file.display());
        for game in read_pgn_games(File::open(pgn_file)?) {
            let mut white = String::new();
            let mut black = String::new();
            let mut opening = String::new();

            for (tag, value) in game?.tags()? {
                match tag.as_str() {
                    "White" => white = value,
                    "Black" => black = value,
                    "TestOpening" => opening = value,
                    _ => {},
                }
            }

            let entry = (white.clone(), black.clone(), opening.clone());
            white_games.remove(&entry);
            let entry = (black.clone(), white.clone(), opening.clone());
            black_games.remove(&entry);
        }
    }

    let mut single_games: Vec<_> = white_games
        .symmetric_difference(&black_games)
        .cloned()
        .collect();
    single_games.shuffle(&mut rand::thread_rng());
    let mut game_pairs: Vec<_> = white_games
        .intersection(&black_games)
        .cloned()
        .collect();
    game_pairs.shuffle(&mut rand::thread_rng());
    let all_games = [single_games, game_pairs].concat();

    let mut game_setup = GameSetup::new();
    game_setup.time_control(TimeControl::Incremental{
            base: Duration::from_secs(60),
            inc: Duration::from_secs(1), });

    for game in all_games {
        let (eng_name, opp_name, opening) = &game;
        game_setup.opening(openings[opening].parse::<MoveSequence>()?);

        // open engine's pgn file
        let pgn_file = paths.games_dir.join(eng_name.to_owned() + ".pgn");
        let mut pgn_file = OpenOptions::new()
            .append(true)
            .create(true) // create if doesn't already exist
            .open(pgn_file)?;

        let eng_cmd = &candidates[eng_name];
        let opp_cmd = &opponents[opp_name];

        println!("{} vs {} ({:#})", eng_name, opp_name, opening);
        if white_games.contains(&game) {
            match play_game(
                eng_name, eng_cmd,
                opp_name, opp_cmd,
                opening, &game_setup)
            {
                Ok(pgn) => {
                    println!("{}", pgn);
                    writeln!(pgn_file, "{}", pgn)?;
                },
                Err(error) => {
                    println!("{}", error);
                    error!("{}", error);
                }
            }
        } else {
            println!("\t*** skipping, already been played ***\n");
        }

        println!("{} vs {} ({:#})", opp_name, eng_name, opening);
        if black_games.contains(&game) {
            match play_game(
                opp_name, opp_cmd,
                eng_name, eng_cmd,
                opening, &game_setup)
            {
                Ok(pgn) => {
                    println!("{}", pgn);
                    writeln!(pgn_file, "{}", pgn)?;
                },
                Err(error) => {
                    println!("{}", error);
                    error!("{}", error);
                }
            }
        } else {
            println!("\t*** skipping, already been played ***\n");
        }

        // TODO:
        // if any input files have changed, re-read them
    }

    Ok(())
}

fn play_game(
    white_name: &str, white_cmd: &[String],
    black_name: &str, black_cmd: &[String],
    opening: &str, game_setup: &GameSetup)
-> std::io::Result<String> {
    let white = Box::new(XboardClient::new(
        &white_cmd[0],
        &white_cmd[1..],
        white_name)?);
    let black = Box::new(XboardClient::new(
        &black_cmd[0],
        &black_cmd[1..],
        black_name)?);

    let mut pgn_tags = HashMap::new();
    pgn_tags.insert("Event".to_owned(), "Automated testing".to_owned());
    if let Ok(hostname) = hostname::get() {
        if let Ok(hostname) = hostname.into_string() {
            pgn_tags.insert("Site".to_owned(), hostname);
        }
    }
    pgn_tags.insert("Date".to_owned(), Local::today().format("%Y.%m.%d").to_string());
    pgn_tags.insert("Round".to_owned(), "-".to_owned());
    pgn_tags.insert("TestOpening".to_owned(), opening.to_owned());

    pgn_tags.insert("White".to_owned(), white_name.to_owned());
    pgn_tags.insert("Black".to_owned(), black_name.to_owned());

    Ok(game_setup.play_game(white, black).0.to_pgn(&pgn_tags))
}

fn read_engine_file(path: &Path) -> Result<HashMap<String, Vec<String>>, Error> {
    let s = read_to_string(path)?;
    Ok(serde_yaml::from_str(&s)?)
}

fn write_engine_file(path: &Path, engines: &HashMap<String, Vec<String>>) -> Result<(), Error> {
    let s = serde_yaml::to_string(engines)?;
    write(path, s)?;
    Ok(())
}

fn read_opening_file(path: &Path) -> Result<HashMap<String, String>, Error> {
    let s = read_to_string(path)?;
    Ok(serde_yaml::from_str(&s)?)
}

fn write_opening_file(path: &Path, openings: &HashMap<String, String>) -> Result<(), Error> {
    let s = serde_yaml::to_string(openings)?;
    write(path, s)?;
    Ok(())
}

fn read_pgn_openings(path: &Path) -> Result<Vec<String>, Error> {
    let mut list = Vec::new();
    let s = read_to_string(path)?;

    let mut seq = String::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('[') {
            if !seq.is_empty() {
                list.push(seq);
                seq = String::new();
            }
            continue;
        } else if line.starts_with("1.") {
            if !seq.is_empty() {
                list.push(seq);
            }
            seq = line.to_string();
        } else {
            seq += " ";
            seq += line;
        }
    }
    if !seq.is_empty() {
        list.push(seq);
    }

    Ok(list)
}

#[derive(Debug)]
struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error { }

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<tinman::chess::Error> for Error {
    fn from(err: tinman::chess::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<tinman::pgn::PgnParseError> for Error {
    fn from(err: tinman::pgn::PgnParseError) -> Self {
        Error(err.to_string())
    }
}
