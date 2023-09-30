use crate::color::print_color;
use crate::color::Color;
use crate::command::Move;

pub type Board = (u64, u64);

const INIT_BLACK: u64 = 0x0000000810000000;
const INIT_WHITE: u64 = 0x0000001008000000;

pub fn init_board() -> Board {
    (INIT_BLACK, INIT_WHITE)
}

const CORNERS: [((i8, i8), (i8, i8)); 4] = [
    ((0, 0), (1, 1)),
    ((7, 7), (-1, -1)),
    ((7, 0), (-1, 1)),
    ((0, 7), (1, -1)),
];

#[inline]
fn has_stone(board: u64, pos: i8) -> bool {
    ((board >> pos) & 1) == 1
}
macro_rules! line {
    ($r:ident, $p:expr, $m:expr, $s:ident, $n:expr) => {
        $r = $m & $s($p, $n);
        $r |= $m & $s($r, $n);
        $r |= $m & $s($r, $n);
        $r |= $m & $s($r, $n);
        $r |= $m & $s($r, $n);
        $r |= $m & $s($r, $n);
    };
}

#[inline]
const fn shl(a: u64, b: u32) -> u64 {
    a << b
}

#[inline]
const fn shr(a: u64, b: u32) -> u64 {
    a >> b
}

pub fn flippable_bits(pboard: u64, oboard: u64) -> u64 {
    let blank = !(pboard | oboard);

    macro_rules! calc {
        ($r: ident, $p:expr, $o: expr, $m:expr, $n:expr, $b: expr) => {
            let m = $o & $m;
            let mut bit;
            line!(bit, $p, m, shl, $n);
            $r |= $b & shl(bit, $n);
            line!(bit, $p, m, shr, $n);
            $r |= $b & shr(bit, $n);
        };
    }

    let mut res = 0;
    calc!(res, pboard, oboard, 0x7e7e7e7e7e7e7e7e, 1, blank);
    calc!(res, pboard, oboard, 0x00ffffffffffff00, 8, blank);
    calc!(res, pboard, oboard, 0x007e7e7e7e7e7e00, 7, blank);
    calc!(res, pboard, oboard, 0x007e7e7e7e7e7e00, 9, blank);

    res
}

pub const fn flip(pboard: u64, oboard: u64, pos: u64) -> u64 {
    macro_rules! calc {
        ($r:ident, $s:expr, $o:expr, $p:expr, $m:expr, $n:expr) => {
            let mask = $o & $m;
            let mut bit;
            line!(bit, $p, mask, shl, $n);
            if $s & shl(bit, $n) != 0 {
                $r |= bit;
            }
            line!(bit, $p, mask, shr, $n);
            if $s & shr(bit, $n) != 0 {
                $r |= bit;
            }
        };
    }

    let mut result = 0;
    calc!(result, pboard, oboard, pos, 0x7e7e7e7e7e7e7e7e, 1);
    calc!(result, pboard, oboard, pos, 0x007e7e7e7e7e7e00, 7);
    calc!(result, pboard, oboard, pos, 0x00ffffffffffff00, 8);
    calc!(result, pboard, oboard, pos, 0x007e7e7e7e7e7e00, 9);
    result
}

pub fn valid_mask(board: &Board, color: Color) -> u64 {
    let pboard: u64;
    let oboard: u64;
    if color {
        pboard = board.0;
        oboard = board.1;
    } else {
        pboard = board.1;
        oboard = board.0;
    }
    flippable_bits(pboard, oboard)
}

pub fn valid_moves(board: &Board, color: Color) -> Vec<(i8, i8)> {
    let mut mask = valid_mask(board, color);
    let mut res = vec![];
    for i in 0..8 {
        for j in 0..8 {
            if mask & 1 == 1 {
                res.push((i, j));
            }
            mask >>= 1;
        }
    }
    res
}

pub fn do_move(board: &mut Board, com: &Move, color: Color) {
    let pboard: &mut u64;
    let oboard: &mut u64;
    if color {
        pboard = &mut board.0;
        oboard = &mut board.1;
    } else {
        pboard = &mut board.1;
        oboard = &mut board.0;
    }
    match com {
        Move::GiveUp => {}
        Move::Pass => {}
        Move::Mv(i, j) => {
            let pos = 1 << (i * 8 + j);
            let mask = flip(*pboard, *oboard, pos);
            *pboard |= mask | pos;
            *oboard &= !mask;
        }
    }
}

pub fn count(board: u64) -> i8 {
    let mut bits = board;
    bits = (bits & 0x5555555555555555) + ((bits >> 1) & 0x5555555555555555);
    bits = (bits & 0x3333333333333333) + ((bits >> 2) & 0x3333333333333333);
    bits = (bits & 0x0F0F0F0F0F0F0F0F) + ((bits >> 4) & 0x0F0F0F0F0F0F0F0F);
    bits = (bits & 0x00FF00FF00FF00FF) + ((bits >> 8) & 0x00FF00FF00FF00FF);
    bits = (bits & 0x0000FFFF0000FFFF) + ((bits >> 16) & 0x0000FFFF0000FFFF);
    bits = (bits & 0x00000000FFFFFFFF) + ((bits >> 32) & 0x00000000FFFFFFFF);
    bits as i8
}

pub fn print_board(board: &Board) {
    println!("{} {}", board.0, board.1);
    println!(" |A B C D E F G H");
    println!("-+----------------");
    for j in 0..8 {
        print!("{}|", j + 1);
        for i in 0..8 {
            let pos = i * 8 + j;
            let is_black = has_stone(board.0, pos);
            let is_white = has_stone(board.1, pos);
            print_color(is_black || is_white, is_black)
        }
        println!();
    }
    println!("  (X: Black,  O: White)");
}

pub fn get_result(board: &Board, color: Color) -> i8 {
    let pboard: u64;
    let oboard: u64;
    if color {
        pboard = board.0;
        oboard = board.1;
    } else {
        pboard = board.1;
        oboard = board.0;
    }
    let my_count = count(pboard);
    let op_count = count(oboard);
    if my_count > op_count {
        1
    } else if my_count < op_count {
        -1
    } else {
        0
    }
}

fn calc_weight(board: u64) -> i32 {
    let mut board = board;
    /*static WEIGHTS: [i32; 64] = [
        120, -20, 20, 5, 5, 20, -20, 120, -20, -40, -5, -5, -5, -5, -40, -20, 20, -5, 15, 3, 3, 15,
        -5, 20, 5, -5, 3, 3, 3, 3, -5, 5, 5, -5, 3, 3, 3, 3, -5, 5, 20, -5, 15, 3, 3, 15, -5, 20,
        -20, -40, -5, -5, -5, -5, -40, -20, 120, -20, 20, 5, 5, 20, -20, 120,
    ];*/
    static WEIGHTS: [i32; 64] = [
        30, -12, 0, -1, -1, 0, -12, 30, -12, -15, -3, -3, -3, -3, -15, -12, 0, -3, 0, -1, -1, 0,
        -3, 0, -1, -3, -1, -1, -1, -1, -3, -1, -1, -3, -1, -1, -1, -1, -3, -1, 0, -3, 0, -1, -1, 0,
        -3, 0, -12, -15, -3, -3, -3, -3, -15, -12, 30, -12, 0, -1, -1, 0, -12, 30,
    ];
    let mut res = 0;
    for i in 0..64 {
        if board & 1 == 1 {
            res += WEIGHTS[i];
        }
        board = board >> 1;
    }
    res
}

/*static WEIGHTS: [i32; 64] = [
    100, -40, 20, 5, 5, 20, -40, 100, -40, -80, -1, -1, -1, -1, -80, -40, 20, -1, 5, 1, 1, 5,
    -1, 20, 5, -1, 1, 0, 0, 1, -1, 5, 5, -1, 1, 0, 0, 1, -1, 5, 20, -1, 5, 1, 1, 5, -1, 20,
    -40, -80, -1, -1, -1, -1, -80, -40, 100, -40, 20, 5, 5, 20, -40, 100,
];*/

pub fn count_stable(board: u64) -> i32 {
    let mut res = 0;
    let mut dup = 0;
    for (corner, d) in CORNERS {
        if !has_stone(board, corner.0 * 8 + corner.1) {
            continue;
        }
        res += 1;
        let mut i = 1;
        while i <= 7 && has_stone(board, (corner.0 + d.0 * i) * 8 + corner.1) {
            i += 1;
        }
        res += i - 1;
        if i == 8 {
            dup += i;
        }
        let mut i = 1;
        while i <= 7 && has_stone(board, corner.0 * 8 + corner.1 + d.1 * i) {
            i += 1;
        }
        res += i - 1;
        if i == 8 {
            dup += i;
        }
        let mut i = 1;
        while i <= 7 && has_stone(board, (corner.0 + d.0 * i) * 8 + corner.1 + d.1 * i) {
            i += 1;
        }
        res += i - 1;
        if i == 8 {
            dup += i;
        }
    }
    (res - dup / 2) as i32
}

pub fn openness(flippable_bits: u64, pboard: u64, oboard: u64) -> i32 {
    let mut neighbors = (flippable_bits << 1) & 0xfefefefefefefefe;
    neighbors |= (flippable_bits >> 1) & 0x7f7f7f7f7f7f7f7f;
    neighbors |= flippable_bits << 8;
    neighbors |= flippable_bits >> 8;
    neighbors |= (flippable_bits << 9) & 0xfefefefefefefefe;
    neighbors |= (flippable_bits >> 9) & 0x7f7f7f7f7f7f7f7f;
    neighbors |= (flippable_bits << 7) & 0x7f7f7f7f7f7f7f7f;
    neighbors |= (flippable_bits >> 7) & 0xfefefefefefefefe;
    neighbors &= !(pboard | oboard);
    count(neighbors) as i32
}

pub fn evaluate(board: &Board, board_p: &Board, color: Color, depth: i8) -> i32 {
    let pboard: u64;
    let oboard: u64;
    let oboard_p: u64;
    if color {
        pboard = board.0;
        oboard = board.1;
        oboard_p = board_p.1;
    } else {
        pboard = board.1;
        oboard = board.0;
        oboard_p = board_p.0;
    }
    if depth <= 4 {
        let p_count = count(pboard) as i32;
        let o_count = count(oboard) as i32;
        (p_count - o_count) * 33554431
    } else {
        let pweight = calc_weight(pboard);
        let oweight = calc_weight(oboard);
        let openess = openness(oboard ^ oboard_p, pboard, oboard);
        let pcandidates = count(flippable_bits(pboard, oboard)) as i32;
        let ocandidates = count(flippable_bits(oboard, pboard)) as i32;
        let pstables = count_stable(pboard);
        let ostables = count_stable(oboard);
        openess * 10
            + (pweight - oweight)
            + 10 * (pcandidates - ocandidates)
            + 50 * (pstables - ostables)
    }
}

#[inline]
const fn flip_vertical_data(data: u64) -> u64 {
    data.swap_bytes()
}
#[inline]
const fn rotate180_data(data: u64) -> u64 {
    data.reverse_bits()
}
#[inline]
const fn flip_diagonal_data(data: u64) -> u64 {
    macro_rules! calc {
        ($r:ident, $m:expr, $n:expr) => {
            let mask = $m & ($r ^ ($r << $n));
            $r ^= mask ^ (mask >> $n);
        };
    }

    let mut result = data;
    calc!(result, 0x0f0f0f0f00000000, 28);
    calc!(result, 0x3333000033330000, 14);
    calc!(result, 0x5500550055005500, 07);
    result
}

pub fn expand(board: &Board) -> Vec<Board> {
    let mut res = vec![];
    res.push(board.clone());

    for board in res.clone() {
        res.push((flip_vertical_data(board.0), flip_vertical_data(board.1)));
    }
    for board in res.clone() {
        res.push((rotate180_data(board.0), rotate180_data(board.1)));
    }
    for board in res.clone() {
        res.push((flip_diagonal_data(board.0), flip_diagonal_data(board.1)));
    }
    res
}
