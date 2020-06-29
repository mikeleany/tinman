//! Tinman is a computer chess engine developed by [Mike Leany](http://www.mikeleany.com/).
//! It is written in Rust and based on his old chess engine, [Vapor](http://vapor.mikeleany.com/),
//! which was written in C.
//!
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! To run the engine, you will probably want some kind of graphical user interface. You will need
//! one that is compatible with the xboard protocol, such as
//! [XBoard/WinBoard](https://www.gnu.org/software/xboard/) or
//! [Arena](http://www.playwitharena.de/). Refer to the documentation for the specific user
//! interface for instructions on how to set up new engines. You will need to give the interface
//! the command to run which, in the simplest case, is just the path of the executable.
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

pub mod pgn;
pub mod protocol;
pub mod engine;
pub mod client;
