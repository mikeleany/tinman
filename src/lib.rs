//! Tinman - A Rusty chess engine.
//!
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! **This documentation is for Tinman's code. If you're looking for documentation regarding
//! installation and execution, see README.md.**
//!
//! Tinman is a chess engine by [Mike Leany](http://www.mikeleany.com/), written in Rust and loosely
//! based on his previous chess engine [Vapor](https://github.com/mikeleany/vapor), which was
//! written in C. It currently supports the Chess Engine Communication Protocol (XBoard/WinBoard).
//! UCI support will be coming in a future release.
//!
//! Tinman uses a workspace for organization, and contains the following packages:
//!
//! *   `tinman` (this package) is the primary package and is the contains the binary crate for the
//!     Tinman engine.
//! *   [`chess`](../chess/index.html) is a library which implements the FIDE Laws of Chess.
//! *   [`protocols`](../protocols/index.html) is a library which provides for communication
//!     between client and engine.
//! *   [`tinman-test`](../tinman_test/index.html) contains a binary crate for a command-line client
//!     used for testing new versions of Tinman against a set of opponent engines.
//!     
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::todo)]
#![warn(clippy::option_unwrap_used, clippy::result_unwrap_used)]

mod engine;
pub use engine::Engine;
