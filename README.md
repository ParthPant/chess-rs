## About The Project

chess-rs is a Chess Engine written from scratch in Rust that runs natively and on web!

Live Demo: [https://parthpant.github.io/chess-rs/](https://parthpant.github.io/chess-rs/)

![screenshot](./images/screenshot.png)

### Features

- [x] Move Generation using BitBoards (<8s perft results at depth 6 for starting position)
- [x] Supports all moves including en-passant, pawn promotion and castling
- [x] GUI gameplay
- [x] Perft Runner
- [x] Egui Integration
- [x] Board Evaluation (Matrial and Positional)
- [x] AI (NegaMax with Quiescence Search)
- [x] Move Ordering (MVV-LVA, Killer & History Heuristics)
- [x] Zobrist Hashing
- [x] Transposition Tables
- [x] Incremental Search Deepening
- [ ] Opening Book


## Getting Started

## Prerequisites

### Rust
You will require the rust toolchain to be installed on your system.

Follow: [Rust Website](https://www.rust-lang.org/tools/install)

Note: If you only want to run natively you are good to go. To build for
WASM you will also need to install the following.

### Web Target
```
rustup target add wasm32-unknown-unknown
```

### Trunk
```
cargo install --locked trunk
```


## Build from source

Simply clone the Git repository and build using the cargo build system
```
git clone https://github.com/ParthPant/chess-rs.git
cd chess-rs
cargo build
```

## Usage

```
# You can either start the Chess Engine with
cargo run

# Or you can run in the web browser with
trunk serve

# Or you can run perft analysis
cargo run -p chrs-perft -- 5 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```


## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

## Contact

Parth Pant - [@PantParth](https://twitter.com/PantParth) - parthpant4@gmail.com

Project Link: [https://github.com/ParthPant/chess-rs.git](https://github.com/ParthPant/chess-rs.git)
