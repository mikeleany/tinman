//! Implements the [Chess Engine Communication Protocol](http://hgm.nubati.net/CECP.html), commonly
//! known as xboard.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fmt;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::num::{ParseIntError, ParseFloatError};
use std::sync::mpsc::TryRecvError;
use std::ffi::OsStr;
use log::{debug, info, error};
use lazy_static::lazy_static;
use regex::{RegexSet, Regex};
use super::{Protocol, Action, SearchAction, Thinking, io};
use chess::game::{Game, TimeControl, GameResult};
use crate::client::{EngineInterface, EngineResponse, EngineError};

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Current state of the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Idle,
    Thinking,
    Pondering(chess::ArcMove),
    Quitting,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Implementation of the xboard protocol
#[derive(Debug)]
pub struct Xboard {
    client: io::Client,

    game: Game,

    state: State,
    color: Option<chess::Color>,
    post_thinking: bool,
    can_ponder: bool,
    max_depth: Option<usize>,

    ponder_hits: usize,
    ponder_total: usize,
}

impl Xboard {
    /// Starts the xboard interface and engine running.
    pub fn new() -> Self {
        Xboard {
            client: io::Client::connect(),
            game: Game::new(),
            state: State::Idle,
            color: Some(chess::Color::Black),
            post_thinking: true,
            can_ponder: true,
            max_depth: None,
            ponder_hits: 0,
            ponder_total: 0,
        }
    }
}

impl Protocol for Xboard {

    fn wait_for_direction(&mut self) -> Action {
        use Command::*;

        while self.state == State::Idle {
            if let Ok(line) = self.client.recv() {
                if let Ok(cmd) = line.parse() {
                    match cmd {
                        Xboard => { },
                        Protover(_) => {
                            use FeatureValue::*;
                            Response::Feature(vec![
                                Feature{name: "done".to_owned(), value: Int(0)}
                            ]).send();
                            Response::Feature(vec![
                                Feature{ name: "sigint".to_owned(), value: Int(0) },
                                Feature{ name: "sigterm".to_owned(), value: Int(0) },
                                Feature{ name: "ping".to_owned(), value: Int(1) },
                                Feature{ name: "setboard".to_owned(), value: Int(1) },
                                Feature{ name: "myname".to_owned(),
                                    value: String("tinman".to_owned()) },
                                Feature{ name: "debug".to_owned(), value: Int(1) },
                                Feature{ name: "nps".to_owned(), value: Int(0) },
                                Feature{ name: "analyze".to_owned(), value: Int(0) },
                                Feature{ name: "memory".to_owned(), value: Int(1) },
                            ]).send();
                            Response::Feature(vec![
                                Feature{name: "done".to_owned(), value: Int(1)}
                            ]).send();
                        },
                        Accepted(_) => { },
                        Rejected(_name) => {
                            // TODO: check if that's a problem
                        },
                        Ping(n) => {
                            Response::Pong(n).send();
                        },
                        Quit => {
                            self.state = State::Quitting;
                            return Action::Quit;
                        },
                        New => {
                            self.game = Game::new();
                            self.color = Some(chess::Color::Black);
                            self.max_depth = None;
                        },
                        Force => {
                            self.color = None;
                        },
                        Go => {
                            self.color = Some(self.game.position().turn());
                            self.state = State::Thinking;
                        },
                        UserMove(move_str) => {
                            match self.game.make_move_from_str(&move_str) {
                                Ok(_) => {
                                    if let Some(result) = self.game.result() {
                                        // TODO: use Response
                                        io::Client::send(&result.to_string());
                                    } else if self.color == Some(self.game.position().turn()) {
                                        self.state = State::Thinking;
                                    }
                                },
                                Err(error) => {
                                    debug!("illegal move {} from {}", move_str, self.game.position());
                                    Response::IllegalMove(move_str, Some(error.to_string())).send();
                                },
                            }
                        },
                        SetBoard(fen) => {
                            match fen.parse() {
                                Ok(pos) => {
                                    let tc = self.game.clock().time_control();
                                    self.game = Game::starting_at(pos);
                                    self.game.set_time_control(tc);
                                },
                                Err(err) => Response::ErrorMessage(line, err.to_string()).send(),
                            }
                        },
                        Draw => {
                            // TODO: consider if draw should be accepted
                        },
                        GameResult{ .. } => {
                            self.color = None;
                        },
                        Undo => {
                            self.game.undo();
                        },
                        Remove => {
                            self.game.undo();
                            self.game.undo();
                        },
                        MoveNow => {
                            // TODO: not valid here
                        },
                        Time(time) => {
                            if let Some(color) = self.color {
                                self.game.clock_mut().set(color, time);
                            } else {
                                let color = self.game.position().turn();
                                self.game.clock_mut().set(color, time);
                            }
                        },
                        OppTime(time) => {
                            if let Some(color) = self.color {
                                self.game.clock_mut().set(!color, time);
                            } else {
                                let color = self.game.position().turn();
                                self.game.clock_mut().set(!color, time);
                            }
                        },
                        Level{ mps, base, inc } => {
                            self.game.set_time_control(match (mps, inc.as_millis()) {
                                (0, 0) => TimeControl::SuddenDeath(base),
                                (0, _) => TimeControl::Incremental{ base, inc },
                                (_, 0) => TimeControl::Session{ base, mps },
                                _ => {
                                    Response::ErrorMessage(line.clone(),
                                        "invalid time control".to_string()).send();
                                    continue
                                },
                            });
                        },
                        SetTime(time) => {
                            self.game.set_time_control(TimeControl::Exact(time));
                        },
                        SetDepth(depth) => {
                            self.max_depth = Some(depth);
                        },
                        Memory(size) => {
                            return Action::HashSize(size * 0x10_0000);
                        },
                        Post => {
                            self.post_thinking = true;
                        },
                        NoPost => {
                            self.post_thinking = false;
                        },
                        Ponder => {
                            self.can_ponder = true;
                        },
                        NoPonder => {
                            self.can_ponder = false;
                        },
                        Hint => {
                            Response::ErrorMessage(line.clone(),
                                "You're on your own on this one".to_string()).send();
                        },
                    }
                } else {
                    Response::ErrorMessage(line.clone(),
                        "unknown or incorrectly formatted command".to_string()).send();
                }
            } else {
                error!("input error");
                self.state = State::Quitting;
            }
        }

        if self.state == State::Quitting {
            Action::Quit
        } else {
            Action::Search
        }
    }

    fn send_move(&mut self, thinking: &Thinking) {
        self.send_thinking(thinking);

        // TODO: make_move_timed
        if let Some(mv) = thinking.best_move() {
            self.game.make_move(mv.clone()).expect("INFALLIBLE");
            Response::Move(format!("{:#}", mv)).send();

            self.state = State::Idle; // default to idle

            if let Some(result) = self.game.result() {
                // TODO: use Response
                io::Client::send(&result.to_string());
            } else if self.can_ponder {
                if let Some(mv) = thinking.ponder_move() {
                    self.state = State::Pondering(mv.clone());
                }
            }
        } else {
            if let Some(result) = self.game.result() {
                // TODO: use Response
                io::Client::send(&result.to_string());
            } else if self.color == Some(self.game.position().turn()) {
                self.state = State::Thinking;
            }
            self.state = State::Idle;
        }
    }

    fn send_thinking(&mut self, thinking: &Thinking) {
        if self.post_thinking {
            let pv_string = if let Some(mv) = self.ponder_move() {
                match thinking.pv() {
                    Some(pv) => format!("({}) {}", mv, pv),
                    None => format!("({})", mv),
                }
            } else {
                match thinking.pv() {
                    Some(pv) => pv.to_string(),
                    None => String::new(),
                }
            };

            Response::ThinkingOutput{
                depth: thinking.depth(),
                score: thinking.score().into(),
                time: thinking.time(),
                nodes: thinking.nodes(),
                pv: pv_string,
            }.send();
        }
    }

    fn send_debug_msg(&mut self, msg: &str) {
        Response::DebugMessage(msg.to_string()).send();
    }

    fn check_input(&mut self) -> Option<SearchAction> {
        use Command::*;

        match self.client.try_recv() {
            Ok(line) => {
                if let Ok(cmd) = line.parse() {
                    match cmd {
                        Ping(n) => {
                            Response::Pong(n).send();
                        },
                        Quit => {
                            self.state = State::Quitting;
                            return Some(SearchAction::Abort);
                        },
                        New => {
                            self.game = Game::new();
                            self.state = State::Idle;
                            self.max_depth = None;
                            return Some(SearchAction::Abort);
                        },
                        Force => {
                            self.color = None;
                            self.state = State::Idle;
                            return Some(SearchAction::Abort);
                        },
                        Go => {
                            // really shouldn't be sent while thinking, but we'll interpret that
                            // as a request to restart the search
                            self.color = Some(self.game.position().turn());
                            self.state = State::Thinking;
                            return Some(SearchAction::Abort);
                        },
                        UserMove(move_str) => {
                            match self.game.make_move_from_str(&move_str) {
                                Ok(_) => {
                                    if let Some(result) = self.game.result() {
                                        // TODO: use Response
                                        io::Client::send(&result.to_string());
                                        self.state = State::Idle;
                                        return Some(SearchAction::Abort);
                                    } else if let State::Pondering(mv) = &self.state {
                                        self.ponder_total += 1;
                                        if mv == self.game.history().last().expect("INFALLIBLE") {
                                            self.ponder_hits += 1;
                                            info!("ponder hit: {}/{} = {}%",
                                                self.ponder_hits, self.ponder_total,
                                                100*self.ponder_hits/self.ponder_total);

                                            self.state = State::Thinking;
                                            return Some(SearchAction::PonderHit);
                                        } else {
                                            info!("ponder miss: {}/{} = {}%",
                                                self.ponder_hits, self.ponder_total,
                                                100*self.ponder_hits/self.ponder_total);

                                            self.state = State::Thinking;
                                            return Some(SearchAction::Abort);
                                        }
                                    } else {
                                        self.state = State::Idle;
                                        return Some(SearchAction::Abort);
                                    }
                                },
                                Err(error) => {
                                    debug!("illegal move {} from {}", move_str, self.game.position());
                                    Response::IllegalMove(move_str, Some(error.to_string())).send();
                                },
                            }
                        },
                        SetBoard(fen) => {
                            match fen.parse() {
                                Ok(pos) => {
                                    let tc = self.game.clock().time_control();
                                    self.game = Game::starting_at(pos);
                                    self.game.set_time_control(tc);
                                },
                                Err(err) => Response::ErrorMessage(line, err.to_string()).send(),
                            }
                            self.state = State::Idle;
                            return Some(SearchAction::Abort);
                        },
                        Draw => {
                            // TODO: consider if draw should be accepted
                        },
                        GameResult{ .. } => {
                            self.color = None;
                            self.state = State::Idle;
                            return Some(SearchAction::Abort);
                        },
                        Undo => {
                            self.game.undo();
                            self.state = State::Idle;
                            return Some(SearchAction::Abort);
                        },
                        Remove => {
                            self.game.undo();
                            self.game.undo();
                            self.state = State::Idle;
                            return Some(SearchAction::Abort);
                        },
                        MoveNow => {
                            self.state = State::Thinking;
                            return Some(SearchAction::Stop);
                        },
                        Time(time) => {
                            if let Some(color) = self.color {
                                self.game.clock_mut().set(color, time);
                            } else {
                                let color = self.game.position().turn();
                                self.game.clock_mut().set(color, time);
                            }
                        },
                        OppTime(time) => {
                            if let Some(color) = self.color {
                                self.game.clock_mut().set(!color, time);
                            } else {
                                let color = self.game.position().turn();
                                self.game.clock_mut().set(!color, time);
                            }
                        },
                        SetTime(time) => {
                            self.game.set_time_control(TimeControl::Exact(time));
                        },
                        SetDepth(_) => {
                            Response::ErrorMessage(line.clone(),
                                "cannot change search depth while thinking".to_string())
                                .send();
                        },
                        Memory(_) => {
                            Response::ErrorMessage(line.clone(),
                                "cannot change hash size in while thinking".to_string())
                                .send();
                        },
                        Post => {
                            self.post_thinking = true;
                        },
                        NoPost => {
                            self.post_thinking = false;
                        },
                        Ponder => {
                            self.can_ponder = true;
                        },
                        NoPonder => {
                            self.can_ponder = false;
                            if let State::Pondering(_) = self.state {
                                self.state = State::Idle;
                                return Some(SearchAction::Abort);
                            }
                        },
                        Hint => {
                            if let State::Pondering(mv) = &self.state {
                                Response::Hint(format!("{:#}", mv)).send();
                            } else {
                                Response::ErrorMessage(line.clone(),
                                    "it's not your turn".to_string()).send();
                            }
                        },
                        _ => { },
                    }
                } else {
                    Response::ErrorMessage(line.clone(),
                        "unknown or incorrectly formatted command".to_string()).send();
                }
            },
            Err(TryRecvError::Disconnected) => {
                error!("lost connection to client");
                self.state = State::Quitting;
                return Some(SearchAction::Abort);
            },
            Err(TryRecvError::Empty) => { },
        }

        None
    }

    fn game(&self) -> &Game {
        &self.game
    }

    fn ponder_move(&self) -> Option<&chess::ArcMove> {
        if let State::Pondering(mv) = &self.state {
            Some(&mv)
        } else {
            None
        }
    }

    fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }
}

impl Default for Xboard {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Implementation of the client-side of the xboard protocol.
#[derive(Debug)]
pub struct XboardClient {
    engine: io::Engine,
    move_count: usize,
    force_mode: bool,
    ponder: bool,
    san: bool,
    usermove: bool,
    ping: usize,
}

impl XboardClient {
    /// Connects to an engine using a new `XboardClient`.
    pub fn new<T, U>(cmd: T, args: &[U], engine_name: &str)
    -> std::io::Result<Self> where T: AsRef<OsStr>, U: AsRef<OsStr> {
        XboardClient {
            engine: io::Engine::launch(cmd, args, engine_name)?,
            move_count: 0,
            force_mode: false,
            ponder: true,
            san: false,
            usermove: false,
            ping: 0,
        }.init()
    }

    /// Enables pondering for the connected engine.
    pub fn enable_pondering(&mut self) {
        self.ponder = true;
    }

    /// Disables pondering for the connected engine.
    pub fn disable_pondering(&mut self) {
        self.ponder = false;
    }

    /// Performs initialization for the engine.
    fn init(mut self) -> std::io::Result<Self> {
        self.send(&Command::Xboard)?;
        self.send(&Command::Protover(2))?;

        let mut stop_time = Some(Instant::now() + Duration::from_secs(10));
        loop {
            let response = if stop_time.is_some() {
                self.engine.try_recv()
            } else {
                self.engine.recv().map_err(|err| {err.into()})
            };
            match response {
                Ok(resp) => {
                    if let Ok(Response::Feature(list)) = resp.parse() {
                        for feature in list {
                            match feature.name.as_str() {
                                "done" => match feature.value {
                                    FeatureValue::Int(0) => {
                                        stop_time = None;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    FeatureValue::Int(1) => {
                                        stop_time = Some(Instant::now());
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    _ => {
                                        self.send(&Command::Rejected(feature.name))?;
                                    },
                                },

                                "san" => match feature.value {
                                    FeatureValue::Int(0) => {
                                        self.san = false;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    FeatureValue::Int(1) => {
                                        self.san = true;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    _ => {
                                        self.send(&Command::Rejected(feature.name))?;
                                    },
                                },

                                "usermove" => match feature.value {
                                    FeatureValue::Int(0) => {
                                        self.usermove = false;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    FeatureValue::Int(1) => {
                                        self.usermove = true;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    _ => {
                                        self.send(&Command::Rejected(feature.name))?;
                                    },
                                },

                                "ping" => match feature.value {
                                    FeatureValue::Int(0) => {
                                        self.ping = 0;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    FeatureValue::Int(1) => {
                                        self.ping = 1;
                                        self.send(&Command::Accepted(feature.name))?;
                                    },
                                    _ => {
                                        self.send(&Command::Rejected(feature.name))?;
                                    },
                                },

                                "sigint" => {
                                    if feature.value == FeatureValue::Int(0) {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        // sigint is not supported
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },

                                // this should only be "times", but some engines use "time" instead
                                "times" | "time" => {
                                    if feature.value == FeatureValue::Int(1) {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        // disabling times is not supported
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },

                                // TODO: make use of these features
                                "setboard" | "memory" | "smp" => {
                                    if let FeatureValue::Int(_) = feature.value {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },
                                // TODO: add support for disabling this feature
                                "reuse" => {
                                    if feature.value == FeatureValue::Int(1) {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },

                                // these features are accepted, but (currently) ignored
                                "debug" | "draw" | "pause" | "nps" | "analyze"
                                | "exclude" | "setscore" | "playother" | "ics"
                                | "name" | "colors" | "sigterm" => {
                                    if let FeatureValue::Int(_) = feature.value {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },

                                // these features are accepted, but (currently) ignored
                                "myname" | "egt" | "option" | "variants" => {
                                    if let FeatureValue::String(_) = feature.value {
                                        self.send(&Command::Accepted(feature.name))?;
                                    } else {
                                        self.send(&Command::Rejected(feature.name))?;
                                    }
                                },

                                // all other features are rejected
                                _ => {
                                    self.send(&Command::Rejected(feature.name))?;
                                },
                            }
                        }
                    }
                },
                Err(TryRecvError::Empty) => {
                    match stop_time {
                        Some(stop_time) if stop_time < Instant::now() => break,
                        _ => std::thread::yield_now(),
                    }
                },
                _ => panic!(),
            }
        }
        // TODO:
        // send any other initialization commands (such as "memory")

        Ok(self)
    }

    /// Send a command to the engne.
    fn send(&mut self, cmd: &Command) -> std::io::Result<()> {
        self.engine.send(&cmd.to_string())
    }

    /// Send a move to the engine.
    fn send_move(&mut self, mv: &chess::ArcMove) -> std::io::Result<()> {
        self.move_count += 1;

        let move_string = if self.san {
            mv.to_string()
        } else {
            format!("{:#}", mv)
        };

        let cmd = Command::UserMove(move_string);

        if self.usermove {
            self.engine.send(&format!("{:#}", cmd))
        } else {
            self.engine.send(&cmd.to_string())
        }
    }

    /// Pings the engine and waits for a response.
    fn ping(&mut self) -> Result<(), EngineError> {
        if self.ping > 0 {
            self.send(&Command::Ping(self.ping))?;
            loop {
                match self.engine.recv()?.parse() {
                    Ok(Response::Pong(n)) => {
                        if n == self.ping {
                            break;
                        } else {
                            return Err(EngineError::ProtocolError);
                        }
                    },
                    Ok(Response::IllegalMove(_, _)) => {
                        return Err(EngineError::RejectedLegalMove);
                    },
                    Ok(Response::GameResult(_, _)) => {
                        return Err(EngineError::FalseResultClaim);
                    },
                    Ok(_) => { /*todo!()*/ }
                    Err(_) => {
                        // ignore unknown responses
                    },
                }
            }
            self.ping += 1;
        }

        Ok(())
    }

    /// Waits for a move (or resign) response from the engine.
    fn wait_for_move(&mut self, pos: &chess::Position, timeout: Duration)
    -> Result<EngineResponse, EngineError> {
        loop {
            match self.engine.recv_timeout(timeout)?.parse() {
                Ok(Response::Move(mv)) => {
                    let mv = mv.parse::<chess::MoveBuilder>()?.validate(pos)?;
                    self.move_count += 1;
                    return Ok(EngineResponse::Move(mv.into()));
                },
                Ok(Response::Resign) => {
                    return Ok(EngineResponse::Resignation);
                },
                Ok(Response::OfferDraw) => {
                },
                Ok(Response::IllegalMove(_, _)) => {
                    return Err(EngineError::RejectedLegalMove);
                },
                Ok(Response::GameResult(_, _)) => {
                    return Err(EngineError::FalseResultClaim);
                },
                Ok(_) => { /*todo!()*/ }
                Err(_) => {
                    // ignore unknown responses
                },
            }
        }
    }
}

impl Drop for XboardClient {
    /// Sends the `quit` command to the attached engine.
    fn drop(&mut self) {
        let _ = self.send(&Command::Quit);
        std::thread::yield_now()
    }
}

impl EngineInterface for XboardClient {
    fn new_game(&mut self, game: &Game) -> Result<(), EngineError> {
        // TODO: include all this in init() and only allow one game per engine invocation
        self.send(&Command::New)?;
        self.send(&Command::Post)?;
        if self.ponder {
            self.send(&Command::Ponder)?;
        } else {
            self.send(&Command::NoPonder)?;
        }
        match game.clock().time_control() {
            TimeControl::Infinite => {
                panic!("infinite time control");
            },
            TimeControl::SuddenDeath(base) => {
                self.send(&Command::Level{ mps: 0, base, inc: Duration::from_secs(0) })?;
            },
            TimeControl::Incremental{ base, inc } => {
                self.send(&Command::Level{ mps: 0, base, inc })?;
            },
            TimeControl::Session{ base, mps } => {
                self.send(&Command::Level{ mps, base, inc: Duration::from_secs(0) })?;
            },
            TimeControl::Exact(time) => {
                self.send(&Command::SetTime(time))?;
            },
        }
        self.force_mode = false;

        if game.history().initial_position().zobrist_key()
            != chess::Position::default().zobrist_key() {
            todo!(); // send position
        }

        self.move_count = 0;
        if !game.history().is_empty() {
            self.send_moves(game)?;
        } else {
            self.ping()?;
        }

        Ok(())
    }

    fn send_moves(&mut self, game: &Game) -> Result<(), EngineError> {
        assert!(self.move_count <= game.history().len());

        if !self.force_mode {
            self.force_mode = true;
            self.send(&Command::Force)?;
        }

        for mv in game.history().iter().skip(self.move_count) {
            self.send_move(mv)?;
        }
        self.move_count = game.history().len();

        self.ping()?;
        Ok(())
    }

    fn go(&mut self, game: &Game) -> Result<EngineResponse, EngineError> {
        let mover = game.position().turn();
        self.send(&Command::Time(game.clock().remaining(mover)))?;
        self.send(&Command::OppTime(game.clock().remaining(!mover)))?;
        self.send(&Command::Go)?;
        self.force_mode = false;

        self.wait_for_move(game.position(), game.time_remaining(mover))
    }

    fn send_move_and_go(&mut self, game: &Game) -> Result<EngineResponse, EngineError> {
        if self.force_mode {
            self.send_moves(game)?;

            self.go(game)
        } else {
            assert_eq!(self.move_count + 1, game.history().len());
            let mover = game.position().turn();
            self.send(&Command::Time(game.clock().remaining(mover)))?;
            self.send(&Command::OppTime(game.clock().remaining(!mover)))?;
            self.send_move(game.history().last().expect("INFALLIBLE"))?;

            self.wait_for_move(game.position(), game.time_remaining(mover))
        }
    }

    fn result(&mut self, game: &Game) -> Result<(), EngineError> {
        self.send(&game.result().expect("game result").into())?;
        self.send(&Command::Force)?;
        self.force_mode = true;
        self.ping()?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Commands which can be sent to the engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Tells the engine to use the xboard protocol.
    ///
    /// ```text
    /// xboard
    /// ```
    Xboard, // initialization

    /// Tells the engine which version of the xboard protocol to use.
    ///
    /// ```text
    /// protover <version>
    /// ```
    ///
    /// `version` is the maximum version number supported by the client.
    Protover(usize), // initialization

    /// Tells the engine that the requested feature is supported.
    ///
    /// ```text
    /// accepted <name>
    /// ```
    ///
    /// `name` is the name of the requested feature.
    Accepted(String), // initialization

    /// Tells the engine that the requested feature is not supported.
    ///
    /// ```text
    /// rejected <name>
    /// ```
    ///
    /// `name` is the name of the requested feature.
    Rejected(String), // initialization

    /// Request that the engine send a "pong" response when it is ready for more input.
    ///
    /// ```text
    /// ping <n>
    /// ```
    ///
    /// `n` is a unique number provided by the client.
    Ping(usize), // idle/pondering

    /// Tells the engine to exit.
    ///
    /// ```text
    /// quit
    /// ```
    Quit, // any, abort search

    /// Begin a new game.
    ///
    /// ```text
    /// new
    /// ```
    New, // any, abort search

    /// Set the engine to receive moves but play neither side.
    ///
    /// ```text
    /// force
    /// ```
    Force, // any, abort search

    /// Sets the engine to begin playing the side currently on move.
    ///
    /// ```text
    /// go
    /// ```
    Go, // idle, change to thinking

    /// Send a move to the engine.
    ///
    /// ```text
    /// <move>
    /// usermove <move>
    /// ```
    ///
    /// The second form is an alternate form which should only be used if requested by the engine.
    ///
    /// `move` is the move to be made.
    UserMove(String), // idle/pondering, change do thinking or abort pondering

    /// Set the board to the given position.
    ///
    /// ```text
    /// setboard <fen>
    /// ```
    ///
    /// `fen` is the position in Forsyth-Edwards Notation.
    SetBoard(String), // any, abort search

    /// Requests a draw.
    ///
    /// ```text
    /// draw
    /// ```
    Draw, // any

    /// Tells the engine that the game has ended with the given result.
    ///
    /// ```text
    /// result <result> [{<reason>}]
    /// ```
    ///
    GameResult{ // any
        /// `result` can be one of the following:
        ///  - 1-0          White wins
        ///  - 0-1          Black wins
        ///  - 1/2-1/2      Draw
        result: String,
        /// An optional plain-text reason for the result (eg. checkmate). It must be
        /// enclosed in curly braces.
        reason: Option<String>
    },

    /// Take back the last move by one side.
    ///
    /// ```text
    /// undo
    /// ```
    Undo, // any, abort search

    /// Take back the last move by each side.
    ///
    /// ```text
    /// remove
    /// ```
    Remove, // any, abort search

    /// Tells the engine to move immediately.
    ///
    /// ```text
    /// ?
    /// ```
    MoveNow, // thinking

    /// Informs the engine of how much time it has remaining.
    ///
    /// ```text
    /// time <remaining>
    /// ```
    ///
    /// `remaining` is the engine's time remaining expressed as an integral number centi-seconds.
    Time(Duration), // idle/pondering

    /// Informs the engine how much time its opponent has remaining.
    ///
    /// ```text
    /// otim <remaining>
    /// ```
    ///
    /// `remaining` is the opponent's time remaining expressed as an integral number centi-seconds.
    OppTime(Duration), // idle/pondering

    /// Sets the initial time controls.
    ///
    /// This command cancels the effect of `Command::SetTime`.
    ///
    /// ```text
    /// level <mps> <base> <inc>
    /// ```
    Level{ // idle
        /// The number of moves per session. It is zero for incremental and sudden death time
        /// controls.
        mps: usize,
        /// The initial amount of time for the game. It can be expressed as a whole number of
        /// minutes or as a number of minutes and seconds in the form `M:SS`.
        base: Duration,
        /// The amount of time added to the player's remaining time after each move. It
        /// is expressed as a number of seconds, which can be a whole number or floating point.
        inc: Duration
    },

    /// Sets the exact amount of time that should be used for each turn.
    ///
    /// This command cancels the effect of `Command::Level`.
    ///
    /// ```text
    /// st <time>
    /// ```
    ///
    /// `time` is the amount of time that should be used for each move expressed in seconds, which
    /// can be a whole number or floating point.
    SetTime(Duration), // idle

    /// Limits the search depth to the depth given.
    ///
    /// ```text
    /// sd <depth>
    /// ```
    ///
    /// `depth` is the maximum depth that the engine should search.
    SetDepth(usize), // idle

    /// Tells the engine how much memory it is allowed to use.
    ///
    /// ```text
    /// memory <n>
    /// ```
    ///
    /// `n` is the maximum amount of memory that should be used by the engine in megabytes.
    Memory(usize), // idle

    /// Turns on thinking output.
    ///
    /// ```text
    /// post
    /// ```
    Post, // any

    /// Turns off thinking output.
    ///
    /// ```text
    /// nopost
    /// ```
    NoPost, // any

    /// Turns on pondering (thinking on the opponent's turn).
    ///
    /// ```text
    /// hard
    /// ```
    Ponder, // any

    /// Turns off pondering (thinking on the opponent's turn).
    ///
    /// ```text
    /// easy
    /// ```
    NoPonder, // any, abort pondering

    /// Asks the engine to suggest a move for the current position.
    ///
    /// ```text
    /// hint
    /// ```
    Hint, // pondering
}

impl From<GameResult> for Command {
    fn from(result: GameResult) -> Self {
        let (result, reason) = match result {
            GameResult::Win(chess::Color::White, reason) =>
                ("1-0".to_owned(), reason.map(|reason| {reason.to_string()})),
            GameResult::Win(chess::Color::Black, reason) =>
                ("0-1".to_owned(), reason.map(|reason| {reason.to_string()})),
            GameResult::Draw(reason) =>
                ("1/2-1/2".to_owned(), reason.map(|reason| {reason.to_string()})),
        };

        Command::GameResult{
            result,
            reason,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Command::*;

        match self {
            Xboard => "xboard".fmt(f),
            Protover(ver) => format!("protover {}", ver).fmt(f),
            Accepted(name) => format!("accepted {}", name).fmt(f),
            Rejected(name) => format!("rejected {}", name).fmt(f),
            Ping(n) => format!("ping {}", n).fmt(f),
            Quit => "quit".fmt(f),
            New => "new".fmt(f),
            Force => "force".fmt(f),
            Go => "go".fmt(f),
            UserMove(mv) => {
                if f.alternate() {
                    format!("usermove {}", mv).fmt(f)
                } else {
                    mv.fmt(f)
                }
            },
            SetBoard(fen) => format!("setboard {}", fen).fmt(f),
            Draw => "draw".fmt(f),
            GameResult{ result, reason: Some(reason) } =>
                format!("result {} {{{}}}", result, reason).fmt(f),
            GameResult{ result, reason: None } => format!("result {}", result).fmt(f),
            Undo => "undo".fmt(f),
            Remove => "remove".fmt(f),
            MoveNow => "?".fmt(f),
            Time(time) => {
                let centis = time.as_millis()/10;
                format!("time {}", centis).fmt(f)
            },
            OppTime(time) => {
                let centis = time.as_millis()/10;
                format!("otim {}", centis).fmt(f)
            },
            Level{ mps, base, inc } => {
                let base = base.as_secs()/60;
                let inc = inc.as_secs();
                format!("level {} {} {}", mps, base, inc).fmt(f)
            },
            SetTime(time) => {
                let time_s = time.as_secs();
                let time_cs = time.subsec_millis()/10;
                if time_cs == 0 {
                    format!("st {}", time_s).fmt(f)
                } else {
                    format!("st {}.{:02}", time_s, time_cs).fmt(f)
                }
            }
            SetDepth(depth) => format!("sd {}", depth).fmt(f),
            Memory(mem) => format!("memory {}", mem).fmt(f),
            Post => "post".fmt(f),
            NoPost => "nopost".fmt(f),
            Ponder => "hard".fmt(f),
            NoPonder => "easy".fmt(f),
            Hint => "hint".fmt(f),
        }
    }
}

impl FromStr for Command {
    type Err = XboardError;

    fn from_str(s: &str) -> Result<Self, XboardError> {
        use Command::*;

        if let Some(ind) = COMMAND_SET.matches(s).iter().next() {
            let args = COMMAND_VEC[ind].captures(s).expect("INFALLIBLE");

            match ind {
                0 => Ok(Xboard),
                1 => {
                    Ok(Protover(args.get(1).expect("INFALLIBLE").as_str().parse()?))
                },
                2 => {
                    Ok(Accepted(args.get(1).expect("INFALLIBLE").as_str().to_string()))
                },
                3 => {
                    Ok(Rejected(args.get(1).expect("INFALLIBLE").as_str().to_string()))
                },
                4 => {
                    Ok(Ping(args.get(1).expect("INFALLIBLE").as_str().parse()?))
                },
                5 => Ok(Quit),
                6 => Ok(New),
                7 => Ok(Force),
                8 => Ok(Go),
                9 => {
                    Ok(UserMove(args.get(1).expect("INFALLIBLE").as_str().to_string()))
                },
                10 => {
                    Ok(SetBoard(args.get(1).expect("INFALLIBLE").as_str().to_string()))
                },
                11 => Ok(Draw),
                12 => {
                    let result = args.get(1).expect("INFALLIBLE").as_str().to_string();
                    let reason = if let Some(reason) = args.get(2) {
                        Some(reason.as_str().to_string())
                    } else {
                        None
                    };

                    Ok(GameResult{ result, reason })
                },
                13 => Ok(Undo),
                14 => Ok(Remove),
                15 => Ok(MoveNow),
                16 => {
                    let time: u64 = args.get(1).expect("INFALLIBLE").as_str().parse()?;
                    let time = Duration::from_millis(time*10);
                    Ok(Time(time))
                },
                17 => {
                    let time: u64 = args.get(1).expect("INFALLIBLE").as_str().parse()?;
                    let time = Duration::from_millis(time*10);
                    Ok(OppTime(time))
                },
                18 => {
                    let mps = args.get(1).expect("INFALLIBLE").as_str().parse()?;
                    let base_m: u64 = args.get(2).expect("INFALLIBLE").as_str().parse()?;
                    let base_s: u64 = if let Some(arg) = args.get(3) {
                        arg.as_str().parse()?
                    } else {
                        0
                    };
                    let inc = args.get(4).expect("INFALLIBLE").as_str().parse()?;
                    let base = Duration::from_secs(base_m*60 + base_s);
                    let inc = Duration::from_secs_f64(inc);
                    Ok(Level{ mps, base, inc })
                },
                19 => {
                    let time = args.get(1).expect("INFALLIBLE").as_str().parse()?;
                    let time = Duration::from_secs_f64(time);
                    Ok(SetTime(time))
                },
                20 => {
                    Ok(SetDepth(args.get(1).expect("INFALLIBLE").as_str().parse()?))
                },
                21 => {
                    Ok(Memory(args.get(1).expect("INFALLIBLE").as_str().parse()?))
                },
                22 => Ok(Post),
                23 => Ok(NoPost),
                24 => Ok(Ponder),
                25 => Ok(NoPonder),
                26 => Ok(Hint),
                _ => unreachable!(),
            }
        } else {
            Err(XboardError)
        }
    }
}

const COMMANDS: [&str; 27] = [
    r"^xboard\b",
    r"^protover\s+(\d+)\b",
    r"^accepted\s+(\w+)\b",
    r"^rejected\s+(\w+)\b",
    r"^ping\s+(\d+)\b",
    r"^quit\b",
    r"^new\b",
    r"^force\b",
    r"^go\b",
    r"^(?:usermove\s+)?([a-h][1-8][a-h][1-8][qrbn]?)\b",
    r"^setboard\s+(.+)\b",
    r"^draw\b",
    r"^result\s+([-/012]+)\b\s*(?:\{([^}]+)\})?",
    r"^undo\b",
    r"^remove\b",
    r"^\?\s*$",
    r"^time\s+(\d+)\b",
    r"^otim\s+(\d+)\b",
    r"^level\s+(\d+)\s+(\d+)(?::(\d\d))?\s+([0-9.]+)\b",
    r"^st\s+([0-9.]+)\b",
    r"^sd\s+(\d+)\b",
    r"^memory\s+(\d+)\b",
    r"^post\b",
    r"^nopost\b",
    r"^hard\b",
    r"^easy\b",
    r"^hint\b",
];

lazy_static! {
    static ref COMMAND_SET: RegexSet = RegexSet::new(&COMMANDS).expect("INFALLIBLE");
    static ref COMMAND_VEC: Vec<Regex> = {
        let mut cmd_vec = Vec::new();
        for cmd in &COMMANDS {
            cmd_vec.push(Regex::new(cmd).expect("INFALLIBLE"));
        }
        cmd_vec
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Responses from the engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// Requests that the client use the given features. Should only be sent at engine startup in
    /// response to the `protover` command.
    ///
    /// ```text
    /// feature NAME=VALUE ...
    /// ```
    ///
    /// NAME is the name of the requested feature.
    ///
    /// VALUE is either an integer or a quoted (") string.
    ///
    /// Any number of NAME=VALUE pairs may be sent in one feature response.
    Feature(Vec<Feature>),

    /// Response to the `ping` command indicating that the engine is ready for the next command.
    ///
    /// ```text
    /// pong N
    /// ```
    ///
    /// N is the unique number given in the `ping` command.
    Pong(usize),

    /// Tells the client that the engine is making the given move.
    ///
    /// ```text
    /// move MOVE
    /// ```
    ///
    /// MOVE is the move to be made.
    Move(String),

    /// Tells the client that the engine is claiming or offering a draw.
    ///
    /// ```text
    /// offer draw
    /// ```
    OfferDraw,

    /// Tells the client that the game has ended with the given result.
    ///
    /// ```text
    /// RESULT [{REASON}]
    /// ```
    ///
    /// RESULT can be one of the following:
    ///
    ///  - 1-0          White wins
    ///  - 0-1          Black wins
    ///  - 1/2-1/2      Draw
    ///
    /// REASON is an optional plain-text reason for the result (eg. checkmate). It must be
    /// enclosed in curly braces.
    GameResult(String, Option<String>),

    /// Tells the client that the engine resigns the game.
    ///
    /// ```text
    /// resign
    /// ```
    Resign,

    /// Tells the client the engine's current line of thinking.
    ///
    /// ```text
    /// <depth> <score> <time> <nodes> <pv>
    /// ```
    ThinkingOutput{
        /// The depth of the current search
        depth: usize,
        /// The value of the current line of thinking
        score: i16,
        /// The amount of time spent thinking on this position (including pondering)
        time: Duration,
        /// The number of nodes searched
        nodes: u64,
        /// One or more moves that make up the principle variation
        pv: String
    },

    /// Response to the hint command, telling the client the current ponder move.
    ///
    /// ```text
    /// Hint: MOVE
    /// ```
    ///
    /// MOVE is the current ponder move.
    Hint(String),

    /// Tells the client that a move received from the client is illegal.
    ///
    /// ```text
    /// Illegal move [(REASON}]: MOVE
    /// ```
    ///
    /// REASON is an optional plain-text reason why the move is illegal (eg. castling through
    /// check). It must be enclosed in parentheses.
    ///
    /// MOVE is the illegal move.
    IllegalMove(String, Option<String>),

    /// Tells the client that the engine doesn't understand the given command.
    ///
    /// ```text
    /// Error (ERRORTYPE): COMMAND
    /// ```
    ///
    /// ERRORTYPE gives the reason for the error (eg. unkown command). It must be enclosed in
    /// parentheses.
    ///
    /// COMMAND is the command that caused the error.
    ErrorMessage(String, String),

    /// A debug message which should be ignored by the client.
    ///
    /// ```text
    /// # MESSAGE
    /// ```
    ///
    /// MESSAGE is the text of the debug message.
    DebugMessage(String),
}

impl Response {
    fn send(&self) {
        io::Client::send(&self.to_string());
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Response::*;

        match self {
            Feature(list) => {
                let mut s = "feature".to_string();
                for feature in list {
                    s += &format!(" {}", feature);
                }
                s.fmt(f)
            },
            Pong(n) => format!("pong {}", n).fmt(f),
            Move(mov) => format!("move {}", mov).fmt(f),
            OfferDraw => "offer draw".fmt(f),
            GameResult(res, Some(reason)) =>
                format!("{} {{{}}}", res, reason).fmt(f),
            GameResult(res, None) => res.fmt(f),
            Resign => "resign".fmt(f),
            ThinkingOutput{ depth, score, time, nodes, pv } =>
                format!("{} {} {} {} {}", depth, score, time.as_millis()/10, nodes, pv).fmt(f),
            Hint(mov) => format!("Hint: {}", mov).fmt(f),
            IllegalMove(mov, Some(reason)) => format!("Illegal move ({}): {}", reason, mov).fmt(f),
            IllegalMove(mov, None) => format!("Illegal move: {}", mov).fmt(f),
            ErrorMessage(cmd, err_type) => format!("Error ({}): {}", err_type, cmd).fmt(f),
            DebugMessage(msg) => format!("# {}", msg).fmt(f),
        }
    }
}

impl FromStr for Response {
    type Err = XboardError;

    fn from_str(s: &str) -> Result<Self, XboardError> {
        use Response::*;

        if let Some(ind) = RESPONSE_SET.matches(s).iter().next() {
            let args = RESPONSE_VEC[ind].captures(s).expect("INFALLIBLE");

            match ind {
                0 => {
                    let args = args.get(1).expect("INFALLIBLE").as_str();
                    let mut list = Vec::new();
                    for feature in FEATURE.captures_iter(args) {
                        let name = feature.get(1).expect("INFALLIBLE").as_str().to_owned();
                        let value = match (feature.get(2), feature.get(3), feature.get(4)) {
                            (Some(val), _, _) => FeatureValue::Int(val.as_str().parse()?),
                            (_, Some(val), _) => FeatureValue::String(val.as_str().to_owned()),
                            (_, _, Some(val)) => FeatureValue::String(val.as_str().to_owned()),
                            _ => unreachable!(),
                        };
                        list.push(self::Feature { name, value });
                    }
                    Ok(Feature(list))
                },
                1 => Ok(Pong(args.get(1).expect("INFALLIBLE").as_str().parse()?)),
                2 => Ok(Move(args.get(1).expect("INFALLIBLE").as_str().to_owned())),
                3 => Ok(OfferDraw),
                4 => {
                    let result = args.get(1).expect("INFALLIBLE").as_str().to_owned();
                    let reason = if let Some(reason) = args.get(2) {
                        Some(reason.as_str().to_owned())
                    } else {
                        None
                    };

                    Ok(GameResult(result, reason))
                },
                5 => Ok(Resign),
                6 => {
                    let depth = args.get(1).expect("INFALLIBLE").as_str().parse()?;
                    let score = args.get(2).expect("INFALLIBLE").as_str().parse()?;
                    let time: u64 = args.get(3).expect("INFALLIBLE").as_str().parse()?;
                    let time = Duration::from_secs(time);
                    let nodes = args.get(4).expect("INFALLIBLE").as_str().parse()?;
                    let pv = args.get(4).expect("INFALLIBLE").as_str().to_owned();
                    Ok(ThinkingOutput{ depth, score, time, nodes, pv })
                },
                7 => Ok(Hint(args.get(1).expect("INFALLIBLE").as_str().to_owned())),
                8 => {
                    let reason = if let Some(reason) = args.get(1) {
                        Some(reason.as_str().to_owned())
                    } else {
                        None
                    };
                    let mov = args.get(2).expect("INFALLIBLE").as_str().to_owned();

                    Ok(IllegalMove(mov, reason))
                },
                9 => {
                    let err_type = args.get(1).expect("INFALLIBLE").as_str().to_owned();
                    let cmd = args.get(2).expect("INFALLIBLE").as_str().to_owned();
                    Ok(ErrorMessage(cmd, err_type))
                },
                10 => Ok(DebugMessage(args.get(1).expect("INFALLIBLE").as_str().to_owned())),
                _ => unreachable!(),
            }
        } else {
            Err(XboardError)
        }
    }
}

const RESPONSES: [&str; 11] = [
    r"^feature\s+(.+)",
    r"^pong\s+(\d+)",
    r"^move\s+(\S+)",
    r"^offer\s+draw\b",
    r"^(1-0|0-1|1/2-1/2)(?:\s+\{([^}]+)\})?", // result
    r"^resign\b",
    r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*(.*)", // thinking output
    r"^Hint:\s+(\S+)",
    r"^Illegal move(?:\s+\(([^)]+)\))?:\s+(\S+)",
    r"^Error\s+\(([^)]+)\):\s+(.+)",
    r"^#\s*(.*)", // debug message
];

lazy_static! {
    static ref RESPONSE_SET: RegexSet = RegexSet::new(&RESPONSES).expect("INFALLIBLE");
    static ref RESPONSE_VEC: Vec<Regex> = {
        let mut resp_vec = Vec::new();
        for response in &RESPONSES {
            resp_vec.push(Regex::new(response).expect("INFALLIBLE"));
        }
        resp_vec
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Value of a protocol feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureValue{
    /// An integer or boolean (0 or 1) value.
    Int(isize),
    /// A string value.
    String(String),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A protocol feature that can be requested by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature{
    name: String,
    value: FeatureValue,
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FeatureValue::*;

        match &self.value {
            Int(val) => format!("{}={}", self.name, val).fmt(f),
            String(val) => format!("{}=\"{}\"", self.name, val).fmt(f),
        }
    }
}

lazy_static! {
    static ref FEATURE: Regex = Regex::new(r#"(\w+)=(?:(\d+)|"([^"]*)"|(\w+))"#).expect("INFALLIBLE");
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Error type for xboard
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct XboardError;

impl From<ParseIntError> for XboardError {
    fn from(_: ParseIntError) -> XboardError {
        XboardError
    }
}

impl From<ParseFloatError> for XboardError {
    fn from(_: ParseFloatError) -> XboardError {
        XboardError
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ***************************************** UNIT TESTS ***************************************** //
////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn format_command() {
        use Command::*;

        assert_eq!(Protover(2).to_string(), "protover 2");
        assert_eq!(Accepted("feature1".to_string()).to_string(), "accepted feature1");
        assert_eq!(Rejected("feature2".to_string()).to_string(), "rejected feature2");
        assert_eq!(Ping(1234).to_string(), "ping 1234");
        assert_eq!(UserMove("g1f3".to_string()).to_string(), "usermove g1f3");
        assert_eq!(
            GameResult{
                result: "1/2-1/2".to_string(),
                reason: Some("stalemate".to_string())
            }.to_string(),
            "result 1/2-1/2 {stalemate}"
        );
        assert_eq!(
            GameResult{
                result: "0-1".to_string(),
                reason: None
            }.to_string(),
            "result 0-1"
        );
        assert_eq!(Time(Duration::from_millis(1024)).to_string(), "time 102");
        assert_eq!(OppTime(Duration::from_millis(50)).to_string(), "otim 5");
        assert_eq!(
            Level{
                mps: 0,
                base: Duration::from_secs(90),
                inc: Duration::from_secs(12)
            }.to_string(),
            "level 0 1:30 12");
        assert_eq!(
            Level{
                mps: 0,
                base: Duration::from_secs(120),
                inc: Duration::from_millis(32)
            }.to_string(),
            "level 0 2:00 0.03");
        assert_eq!(SetTime(Duration::from_secs(5)).to_string(), "st 5");
        assert_eq!(SetTime(Duration::from_millis(10)).to_string(), "st 0.01");
        assert_eq!(SetDepth(12).to_string(), "sd 12");
        assert_eq!(Memory(512).to_string(), "memory 512");
    }

    #[test]
    fn parse_command() {
        use Command::*;

        assert_eq!(Ok(Protover(2)), "protover 2".parse());
        assert_eq!(Ok(Accepted("feature1".to_string())), "accepted feature1".parse());
        assert_eq!(Ok(Rejected("feature2".to_string())), "rejected feature2".parse());
        assert_eq!(Ok(Ping(1234)), "ping 1234".parse());
        assert_eq!(Ok(UserMove("g1f3".to_string())), "usermove g1f3".parse());
        assert_eq!(Ok(UserMove("a7a8q".to_string())), "a7a8q".parse());
        assert_eq!(Ok(
            GameResult{
                result: "1/2-1/2".to_string(),
                reason: Some("stalemate".to_string())
            }),
            "result 1/2-1/2 {stalemate}".parse()
        );
        assert_eq!(Ok(
            GameResult{
                result: "0-1".to_string(),
                reason: None
            }),
            "result 0-1".parse()
        );
        assert_eq!(Ok(Time(Duration::from_millis(1020))), "time 102".parse());
        assert_eq!(Ok(OppTime(Duration::from_millis(50))), "otim 5".parse());
        assert_eq!(Ok(
            Level{
                mps: 0,
                base: Duration::from_secs(90),
                inc: Duration::from_secs(12)
            }),
            "level 0 1:30 12".parse()
        );
        assert_eq!(Ok(
            Level{
                mps: 0,
                base: Duration::from_secs(120),
                inc: Duration::from_millis(32)
            }),
            "level 0 2 0.032".parse()
        );
        assert_eq!(Ok(SetTime(Duration::from_secs(5))), "st 5".parse());
        assert_eq!(Ok(SetTime(Duration::from_millis(10))), "st 0.01".parse());
        assert_eq!(Ok(SetDepth(12)), "sd 12".parse());
        assert_eq!(Ok(Memory(512)), "memory 512".parse());
    }

    #[test]
    fn format_response() {
        use Response::*;

        assert_eq!(Pong(512).to_string(), "pong 512");
        assert_eq!(Move("g1f3".to_string()).to_string(), "move g1f3");
        assert_eq!(GameResult("1/2-1/2".to_string(), Some("stalemate".to_string())).to_string(),
            "1/2-1/2 {stalemate}");
        assert_eq!(GameResult("0-1".to_string(), None).to_string(), "0-1");
        assert_eq!(Hint("g1f3".to_string()).to_string(), "Hint: g1f3");
        assert_eq!(IllegalMove("e1g1".to_string(),
            Some("castling through check".to_string())).to_string(),
            "Illegal move (castling through check): e1g1");
        assert_eq!(IllegalMove("g1f3".to_string(), None).to_string(), "Illegal move: g1f3");
        assert_eq!(ErrorMessage("foo".to_string(), "unknown command".to_string()).to_string(),
            "Error (unknown command): foo");
        assert_eq!(DebugMessage("message".to_string()).to_string(), "# message");
    }

    #[test]
    fn parse_response() {
        use Response::*;

        assert_eq!(Ok(Pong(512)), "pong 512".parse());
        assert_eq!(Ok(Move("g1f3".to_string())), "move g1f3".parse());
        assert_eq!(Ok(GameResult("1/2-1/2".to_string(), Some("stalemate".to_string()))),
            "1/2-1/2 {stalemate}".parse());
        assert_eq!(Ok(GameResult("0-1".to_string(), None)), "0-1".parse());
        assert_eq!(Ok(Hint("g1f3".to_string())), "Hint: g1f3".parse());
        assert_eq!(Ok(IllegalMove("e1g1".to_string(),
            Some("castling through check".to_string()))),
            "Illegal move (castling through check): e1g1".parse());
        assert_eq!(Ok(IllegalMove("g1f3".to_string(), None)), "Illegal move: g1f3".parse());
        assert_eq!(Ok(ErrorMessage("foo".to_string(), "unknown command".to_string())),
            "Error (unknown command): foo".parse());
        assert_eq!(Ok(DebugMessage("message".to_string())), "# message".parse());
    }
}
