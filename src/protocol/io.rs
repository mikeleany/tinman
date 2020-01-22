//! Handles the engine's input and output with the client.
//
//  Copyright 2019 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::io::stdin;
use std::thread;
use std::sync::mpsc::*;
use log::{info, error};

/// Provides a pollable interface with the client using stdin and stdout. All input and output is
/// logged using the log crate (assuming a logger is set up).
#[derive(Debug)]
pub struct Client(Receiver<String>);

impl Client {
    /// Creates and returns a new interface.
    pub fn connect() -> Self {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            Self::thread(sender);
        });

        Self(receiver)
    }

    /// Retrieves a message from the client. Blocks until a message is received.
    pub fn recv(&self) -> Result<String, RecvError> {
        self.0.recv()
    }

    /// Tries to retrieve a message from the client, but does not block if a message is not
    /// available.
    pub fn try_recv(&self) -> Result<String, TryRecvError> {
        self.0.try_recv()
    }

    /// Sends a message to the client.
    pub fn send(s: &str) {
        println!("{}", s);
        info!("<engine>: {}", s);
    }

    /// A function run in a separate thread to get input from stdin.
    ///
    /// # Panics
    ///
    /// Panics if reading from stdin fails for any reason.
    fn thread(sender: Sender<String>) {
        let stdin = stdin();

        loop {
            let mut line = String::new();

            match stdin.read_line(&mut line) {
                Ok(_) => { },
                Err(err) => {
                    error!("io error: {}", err);
                    panic!("io error: {}", err);
                },
            }

            let line = line.trim().to_string();
            info!("<client>: {}", line);
            match sender.send(line) {
                Ok(_) => { },
                Err(err) => {
                    error!("internal error: {}", err);
                    panic!("internal error: {}", err);
                },
            }
        }
    }
}
