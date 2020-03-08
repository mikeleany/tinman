//! Tools for reading and parsing PGN files.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fmt;
use std::io;
use std::io::{Read, BufRead, BufReader};
use std::collections::HashMap;
use crate::chess;
use chess::game::{Game, MoveSequence, GameResult};

pub fn read_pgn_games<R: Read>(reader: R) -> ReadPgnGames<R> {
    ReadPgnGames{ reader: BufReader::new(reader), buffer: String::new() }
}

pub struct ReadPgnGames<R: Read> {
    reader: BufReader<R>,
    buffer: String,
}

impl<R: Read> Iterator for ReadPgnGames<R> {
    type Item = io::Result<PgnParser>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tags = Vec::new();
        let mut move_text = String::new();

        loop {
            let s = self.buffer.trim();

            if s.starts_with('[') {
                if move_text.is_empty() {
                    tags.push(s.to_owned());
                    self.buffer = String::new();
                } else {
                    return Some(Ok(PgnParser{ tags, move_text }));
                }
            } else if !s.is_empty() {
                move_text += " ";
                move_text += s;
            }

            self.buffer.clear();
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => {
                    if tags.is_empty() && move_text.is_empty() {
                        return None;
                    } else {
                        return Some(Ok(PgnParser{ tags, move_text }));
                    }
                },
                Err(error) => return Some(Err(error)),
                _ => {},
            }
        }
    }
}

pub struct PgnParser {
    tags: Vec<String>,
    move_text: String,
}

impl PgnParser {
    pub fn tag_text(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn tags(&self) -> Result<HashMap<String, String>, PgnParseError> {
        let mut tags = HashMap::new();
        for tag in &self.tags {
            let split: Vec<_> = tag
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .trim_end_matches('"')
                .splitn(2, " \"")
                .collect();

            if split.len() == 2 {
                tags.insert(split[0].to_owned(), split[1].to_owned());
            } else {
                return Err(PgnParseError);
            }
        }

        Ok(tags)
    }

    pub fn move_text(&self) -> &str {
        &self.move_text
    }

    pub fn parse_moves(&self) -> chess::Result<MoveSequence> {
        self.move_text.parse()
    }

    pub fn parse_result(&self) -> Result<Option<GameResult>, PgnParseError> {
        todo!()
    }

    pub fn parse_game(&self) -> Result<Game, PgnParseError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct PgnParseError;

impl fmt::Display for PgnParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "PGN parse error".fmt(f)
    }
}

impl std::error::Error for PgnParseError { }
