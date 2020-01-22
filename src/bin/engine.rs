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
