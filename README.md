## About The Project

chess-rs is a Chess Engine written from scratch in Rust.

![screenshot](./screenshot.png)

### Features

This section should list any major features of chess-rs

- [x] Move Generation using BitBoards (~17s perft results at depth 6 for starting position)
- [x] Supports all moves including en-passant, pawn promotion and castling
- [x] GUI gameplay
- [x] Perft Runner
- [x] Egui Integration
- [ ] Move Evaluation
- [ ] Zobrist Hashing
- [ ] AI (aims to be modular)
- [ ] UCI Interface


## Getting Started

## Prerequisites
You will require the rust toolchain to be installed on your system.

Follow: [Rust Website](https://www.rust-lang.org/tools/install)

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
cargo run -p chrs-core

# Or you can run perft analysis
cargo run -p chrs-perft -- 5 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```


## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

## Contact

Parth Pant - [@PantParth](https://twitter.com/PantParth) - parthpant4@gmail.com

Project Link: [https://github.com/ParthPant/chess-rs.git](https://github.com/ParthPant/chess-rs.git)