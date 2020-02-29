//! Handles the engine's input and output with the client.
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::io::stdin;
use std::io::{Write, BufRead, BufReader};
use std::thread;
use std::sync::mpsc::*;
use std::process::{Command, Stdio, Child, ChildStdout, ChildStdin};
use std::time::{Duration, Instant};
use std::ffi::OsStr;
use log::{info, error};

////////////////////////////////////////////////////////////////////////////////////////////////////
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
                Ok(0) => {
                    info!("client at EOF");
                    return;
                },
                Ok(_) => {
                    let line = line.trim().to_string();
                    info!("<client>: {}", line);
                    match sender.send(line) {
                        Ok(_) => { },
                        Err(err) => {
                            error!("internal error: {}", err);
                            panic!("internal error: {}", err);
                        },
                    }
                },
                Err(err) => {
                    error!("io error: {}", err);
                    panic!("io error: {}", err);
                },
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Provides a means of communication with an engine.
#[derive(Debug)]
pub struct Engine {
    id: String,
    recv: Receiver<String>,
    send: ChildStdin,
    proc: Child,
}

impl Engine {
    /// Launches an engine using the given command. Returns an interface to communicate with the
    /// engine.
    pub fn launch<T, U>(cmd: T, args: &[U], id: &str)
    -> std::io::Result<Self> where T: AsRef<OsStr>, U: AsRef<OsStr> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let child_stdout = child.stdout.expect("INFALLIBLE");
        child.stdout = None;
        let child_stdin = child.stdin.expect("INFALLIBLE");
        child.stdin = None;

        let (sender, receiver) = channel();

        let owned_id = id.to_owned();
        thread::spawn(move || {
            Self::thread(child_stdout, sender, owned_id);
        });

        Ok(Engine {
            id: id.to_owned(),
            recv: receiver,
            send: child_stdin,
            proc: child,
        })
    }

    /// Retrieves a message from the engine. Blocks until a message is received.
    pub fn recv(&self) -> Result<String, RecvError> {
        self.recv.recv()
    }

    /// Tries to retrieve a message from the engine, but does not block if a message is not
    /// available.
    pub fn try_recv(&self) -> Result<String, TryRecvError> {
        self.recv.try_recv()
    }

    /// Sends a message to the engine.
    pub fn send(&mut self, s: &str) -> std::io::Result<()> {
        writeln!(self.send, "{}", s)?;
        info!("<to {}>: {}", self.id, s);

        Ok(())
    }

    /// A function run in a separate thread to get input from the engine.
    ///
    /// # Panics
    ///
    /// Panics if reading fails for any reason.
    fn thread(engine: ChildStdout, sender: Sender<String>, id: String) {
        let mut engine = BufReader::new(engine);

        loop {
            let mut line = String::new();

            match engine.read_line(&mut line) {
                Ok(0) => {
                    info!("{} at EOF", id);
                    return;
                },
                Ok(_) => {
                    let line = line.trim().to_string();
                    info!("<from {}>: {}", id, line);
                    match sender.send(line) {
                        Ok(_) => { },
                        Err(err) => {
                            error!("internal error: {}", err);
                            panic!("internal error: {}", err);
                        },
                    }
                },
                Err(err) => {
                    error!("io error: {}", err);
                    panic!("io error: {}", err);
                },
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let kill_time = Instant::now() + Duration::from_secs(1);

        while Instant::now() < kill_time {
            if let Ok(Some(_)) = self.proc.try_wait() {
                return;
            }
            std::thread::yield_now()
        }

        self.proc.kill();
        while self.proc.wait().is_err() { }
    }
}
