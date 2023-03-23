use chrs::data::BoardConfig;
use chrs::generator::MoveGenerator;
use std::time::Instant;

const RES: [usize; 11] = [
    1,
    20,
    400,
    8902,
    197281,
    4865609,
    119060324,
    3195901860,
    84998978956,
    2439530234167,
    69352859712417,
];

fn perft_impl(depth: usize, config: &mut BoardConfig, gen: &MoveGenerator, divide: bool) -> usize {
    if depth == 0 {
        return 1;
    }

    let side = config.get_active_color();
    let moves = gen.gen_all_moves(side, config);

    if depth == 1 {
        return moves.len();
    }

    let mut count = 0;
    for m in moves.iter() {
        if let Some(commit) = config.make_move(*m) {
            let c = perft_impl(depth - 1, config, gen, false);
            if divide {
                println!("{}: {}", commit.m, c);
            }
            count += c;
            config.undo_commit(&commit);
        }
    }

    count
}

#[test]
fn perft() {
    // let mut config = BoardConfig::default();
    let mut config =
        BoardConfig::from_fen_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    // let mut config = BoardConfig::from_fen_str(
    //     "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
    // );
    let gen = MoveGenerator::default();

    let d = 3;
    let t = Instant::now();
    let nodes = perft_impl(d, &mut config, &gen, true);
    let e = t.elapsed();
    // assert_eq!(nodes, RES[d]);
    println!("Results for depth {}: {} leafs, {:?}", d, nodes, e);
}
