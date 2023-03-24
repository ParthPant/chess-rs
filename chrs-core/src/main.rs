#![allow(warnings, unused)]

use pixels::Error;
use pretty_env_logger;

use chrs_core::app::App;

fn main() -> Result<(), Error> {
    // std::env::set_var("RUST_LOG", "chrs=debug");
    pretty_env_logger::init();
    App::run()
}
