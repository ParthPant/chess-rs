#![allow(warnings, unused)]
mod app;
mod board;
mod cache;
mod ui;

use pixels::Error;
use pretty_env_logger;

use app::App;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();
    App::run()
}
