//! Tinman is a 64-bit computer chess engine developed by [Mike Leany](http://www.mikeleany.com/).
//! It is a re-write of my old chess engine, [Vapor](http://vapor.mikeleany.com/), into the Rust
//! programming language.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! # Execution
//! To run the engine, you will probably want some kind of user interface. Once the arbiter tool is
//! completed, it can be used to run automated matches without a graphical interface. If you want
//! a graphical user interface, for instance to play human vs computer game, you will need one that
//! is compatible with the xboard protocol, such as
//! [XBoard/WinBoard](https://www.gnu.org/software/xboard/) or
//! [Arena](http://www.playwitharena.de/). Refer to the documentation for the specific user
//! interface for instructions on how to set up new engines. You will need to give the interface
//! the command to run which is simply the path to the executable.
//!
//! # Additional Tools
//!  - `arbiter` will be a command-line tool meant to facilitate games between two chess engines.
//!  - [`count`](../count/index.html) is a tool which counts the variations from a position to a
//!    specified depth.
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

pub mod chess;
pub mod engine;
pub mod protocol;
