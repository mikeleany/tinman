//! A utility for automatically testing chess engines.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::path::{Path, PathBuf};
use std::fs::{read_to_string, write, create_dir, OpenOptions};
use std::io::Write;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use clap::{App, Arg, SubCommand, AppSettings, crate_version};
use rand::Rng;
use chrono::Local;
use tinman::protocol::xboard::XboardClient;
use tinman::client::{EngineInterface, GameSetup};
use tinman::chess::game::{MoveSequence, TimeControl};

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
                .about("Runs the tests"))
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
    let candidates_file = dir.join("candidates.yaml");
    let opponents_file = dir.join("opponents.yaml");
    let opening_file = dir.join("openings.yaml");
    let bin_dir = dir.join("bin");
    let games_dir = dir.join("games");

    match matches.subcommand() {
        ("init", Some(_matches)) => {
            if !dir.as_os_str().is_empty() && !dir.is_dir() {
                println!("Creating '{}'.", dir.display());
                create_dir(dir)?;
            } else if candidates_file.exists() {
                return Err(Error(format!("'{}' already exists", candidates_file.display())));
            } else if opponents_file.exists() {
                return Err(Error(format!("'{}' already exists", opponents_file.display())));
            } else if opening_file.exists() {
                return Err(Error(format!("'{}' already exists", opening_file.display())));
            }

            if !bin_dir.is_dir() {
                println!("Creating '{}'.", bin_dir.display());
                create_dir(bin_dir)?;
            }
            if !games_dir.is_dir() {
                println!("Creating '{}'.", games_dir.display());
                create_dir(games_dir)?;
            }

            let engines: HashMap<String, Vec<String>> = HashMap::new();
            println!("Creating '{}'.", candidates_file.display());
            write_engine_file(&candidates_file, &engines)?;
            println!("Creating '{}'.", opponents_file.display());
            write_engine_file(&opponents_file, &engines)?;
            let openings: HashMap<String, String> = HashMap::new();
            println!("Creating '{}'.", opening_file.display());
            write_opening_file(&opening_file, &openings)?;
        },
        ("add", Some(matches)) => {
            let bin = matches.value_of("bin").unwrap().to_string();
            let mut cmd = vec![ bin ];
            if let Some(args) = matches.values_of("args") {
                cmd.extend(args.map(|s| { s.to_string() }));
            }

            let name = matches.value_of("name")
                .unwrap_or(&PathBuf::from(&cmd[0]).file_name().unwrap().to_string_lossy())
                .to_string();

            let file = if matches.is_present("opponent") { opponents_file } else { candidates_file };
            let mut engines = read_engine_file(&file)?;
            engines.insert(name, cmd);
            write_engine_file(&file, &engines)?;
        },
        ("remove", Some(matches)) => {
            let name = matches.value_of("name").unwrap();
            let file = if matches.is_present("opponent") { opponents_file } else { candidates_file };
            let mut engines = read_engine_file(&file)?;
            engines.remove(name);
            write_engine_file(&file, &engines)?;
        },
        ("openings", Some(matches)) => {
            let mut openings = read_opening_file(&opening_file)?;
            for file in matches.values_of("files").unwrap() {
                for opening in read_pgn_openings(&PathBuf::from(file))? {
                    let opening = opening.parse::<MoveSequence>().unwrap();
                    let final_pos = opening.final_position().to_string();
                    if !openings.contains_key(&final_pos) {
                        openings.insert(final_pos, opening.to_string());
                    } else {
                        // TODO: warning
                    }
                }
            }

            write_opening_file(&opening_file, &openings)?;
        },
        ("run", Some(_matches)) => {
            let candidates: HashMap<String, Vec<String>> = read_engine_file(&candidates_file)?;
            let opponents: HashMap<String, Vec<String>> = read_engine_file(&opponents_file)?;
            let openings = read_opening_file(&opening_file)?;

            // create list of games to be played
            let mut game_pairs = Vec::new();
            for eng in candidates.keys() {
                for opp in opponents.keys() {
                    for opening in openings.keys() {
                        game_pairs.push((eng, opp, opening));
                    }
                }
            }

            // TODO:
            // determine games played from output pgn files and remove them from the list
            // if any pairs of games are incomplete, play the remaining games

            let mut game_setup = GameSetup::new();
            game_setup.time_control(TimeControl::Incremental{
                    base: Duration::from_secs(60),
                    inc: Duration::from_secs(1), });

            while !game_pairs.is_empty() {
                // randomly select a pair of games to play
                let (eng_name, opp_name, opening) = game_pairs.swap_remove(
                    rand::thread_rng().gen_range(0, game_pairs.len()));
                game_setup.opening(openings[opening].parse::<MoveSequence>().unwrap());

                let candidate = &candidates[eng_name];
                let opponent = &opponents[opp_name];
                let mut eng = XboardClient::new(
                    &candidate[0],
                    &candidate[1..],
                    &eng_name).unwrap();
                let mut opp = XboardClient::new(
                    &opponent[0],
                    &opponent[1..],
                    &opp_name).unwrap();

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

                pgn_tags.insert("White".to_owned(), eng_name.to_owned());
                pgn_tags.insert("Black".to_owned(), opp_name.to_owned());
                println!("{} vs {} ({:#})", eng_name, opp_name, opening);
                let pgn_white = game_setup.play_game(&mut eng, &mut opp).to_pgn(&pgn_tags);
                println!("{}", pgn_white);

                pgn_tags.insert("White".to_owned(), opp_name.to_owned());
                pgn_tags.insert("Black".to_owned(), eng_name.to_owned());
                println!("{} vs {} ({:#})", opp_name, eng_name, opening);
                let pgn_black = game_setup.play_game(&mut opp, &mut eng).to_pgn(&pgn_tags);
                println!("{}", pgn_black);

                // write games to engine's pgn file
                let pgn_file = games_dir.join(eng_name).with_extension("pgn");
                let mut pgn_file = OpenOptions::new()
                    .append(true)
                    .create(true) // create if doesn't already exist
                    .open(pgn_file)?;
                write!(pgn_file, "{}\n{}\n", pgn_white, pgn_black);

                // TODO:
                // if any input files have changed, re-read them
            }
        },
        _ => unreachable!(),
    }

    Ok(())
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
