use crate::data::piece::Color;

pub const NOT_A_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 0 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_AB_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 0 && i % 8 != 1 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_H_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 7 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_GH_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 7 && i % 8 != 6 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const WP_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_pawn_attack(Color::White, i);
        i += 1;
    }
    table
};

pub const BP_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_pawn_attack(Color::Black, i);
        i += 1;
    }
    table
};

pub const N_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_knight_attack(i);
        i += 1;
    }
    table
};

pub const K_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_king_attack(i);
        i += 1;
    }
    table
};

const fn generate_pawn_attack(side: Color, sq: usize) -> u64 {
    let b = 1 << sq;
    match side {
        Color::White => ((b << 7) & NOT_H_FILE) | ((b << 9) & NOT_A_FILE),
        Color::Black => ((b >> 7) & NOT_A_FILE) | ((b >> 9) & NOT_H_FILE),
    }
}

const fn generate_knight_attack(sq: usize) -> u64 {
    let mut b = 1 << sq;
    // << up, >> down
    b = (b << 6) | (b << 15) | (b << 10) | (b << 17) | (b >> 6) | (b >> 15) | (b >> 10) | (b >> 17);

    if sq % 8 == 0 || sq % 8 == 1 {
        b &= NOT_GH_FILE;
    }

    if sq % 8 == 7 || sq % 8 == 6 {
        b &= NOT_AB_FILE;
    }

    b
}

const fn generate_king_attack(sq: usize) -> u64 {
    let mut b = 1 << sq;
    // << up, >> down
    b = (b << 8) | (b >> 8) | (b >> 1) | (b << 1) | (b >> 9) | (b << 9) | (b >> 7) | (b << 7);

    if sq % 8 == 0 {
        b &= NOT_H_FILE;
    }

    if sq % 8 == 7 {
        b &= NOT_A_FILE;
    }

    b
}
