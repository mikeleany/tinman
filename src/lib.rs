//! Tinman is a 64-bit computer chess engine developed by [Mike Leany](http://www.mikeleany.com/).
//! It is a re-write of my old chess engine, [Vapor](http://vapor.mikeleany.com/), into the Rust
//! programming language.
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! # The Executable
//! The executable has not yet been developed. I am currently working on developing the core
//! functionality. To learn more about that, see the modules below.
//!
//! # Additional Tools
//!  - `arbiter` will be a command-line tool meant to facilitate games between two chess engines.
//!  - [`count`](../count/index.html) is a tool which counts the variations from a position to a
//!    specified depth.
////////////////////////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs, missing_debug_implementations, unused_extern_crates)]
#![warn(clippy::unimplemented, clippy::option_unwrap_used, clippy::result_unwrap_used)]

pub mod chess;
pub mod engine;
pub mod protocol;
