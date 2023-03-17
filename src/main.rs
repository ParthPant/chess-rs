#![allow(dead_code)]

mod app;
mod board;
mod cache;
mod data;
mod generator;
mod ui;

use pixels::Error;
use pretty_env_logger;

use app::App;

fn main() -> Result<(), Error> {
    // std::env::set_var("RUST_LOG", "chrs=debug");
    pretty_env_logger::init();
    App::run()
}
