//! The tinman chess engine.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::fs::File;
use simplelog::{WriteLogger, LevelFilter, Config};
use tinman::engine::Engine;
use tinman::protocol::xboard::Xboard;

fn main() {
    let _ = WriteLogger::init(
        LevelFilter::Debug, // TODO: allow this to be set on the command line
        Config::default(),
        File::create("tinman.log").unwrap() // TODO: allow this to be set on the command line
    );

    Engine::new(Xboard::new()).run();
}
