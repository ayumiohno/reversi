use crate::color::Color;
use crate::command::Move;
use crate::database::DATABASE;
use crate::play::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const INF: i32 = i32::MAX;
const THREAD_NUM: usize = 4;
const DEPTH: i8 = 10;
static mut COUNT: u8 = 60;
static mut COLOR: bool = false;
static mut TIME_LIMIT: SystemTime = SystemTime::UNIX_EPOCH;
static mut IS_TIMEOUT: bool = false;
static mut IS_FIRST_END: bool = false;

pub fn set_time_remain(time: u64) {
    unsafe {
        let limit = COUNT as u64 * 900 + 5000;
        let remain = if time > limit { time - limit } else { 0 };
        println!("remain: {}", remain);
        TIME_LIMIT = SystemTime::now() + Duration::from_millis(remain);
    }
}

pub fn init_ai(color: bool) {
    unsafe {
        COUNT = 64;
        COLOR = color;
        IS_TIMEOUT = false;
        IS_FIRST_END = false;
    }
}

fn read_final(board: &Board, color: Color, is_passed: bool) -> Option<i8> {
    if unsafe { IS_TIMEOUT } {
        println!("EXCEEDED");
        return None;
    }
    let valid_moves = valid_moves(board, color);
    if valid_moves.is_empty() {
        let res;
        if is_passed {
            res = get_result(board, color);
        } else {
            res = -read_final(board, !color, true)?;
        }
        return Some(res);
    }
    let mut nodes = vec![];
    for action in valid_moves {
        let mut n_board = board.clone();
        do_move(&mut n_board, &Move::Mv(action.0, action.1), color);
        nodes.push((calc_move_ordering_value(&n_board, color), n_board));
    }
    nodes.sort_by(|a, b| b.0.cmp(&a.0));
    let mut max_res = -1;
    for (_, n_board) in nodes {
        let res = -read_final(&n_board, !color, false)?;
        if res == 1 {
            return Some(1);
        }
        if res > max_res {
            max_res = res;
        }
    }
    Some(max_res)
}

fn read_final_action(color: Color, scores: &Vec<(i32, (i8, i8), Board)>) -> (i8, i8) {
    if scores.is_empty() {
        return (-1, -1);
    }
    let max_res = Arc::new(Mutex::new(-1));
    let best_action = Arc::new(Mutex::new(scores.first().unwrap().1));
    let mut handles = vec![];

    let len = scores.len();
    for i in 0..THREAD_NUM {
        let mut chunk = vec![];
        for j in (i..len).step_by(THREAD_NUM) {
            chunk.push(*scores.get(j).unwrap());
        }
        let max_res = max_res.clone();
        let best_action = best_action.clone();
        handles.push(thread::spawn(move || {
            for (_, action, n_board) in &chunk {
                let res_val = read_final(&n_board, !color, false);
                let res;
                match res_val {
                    Some(val) => res = -val,
                    None => break,
                }
                let mut max_res_p = max_res.lock().unwrap();
                if res > *max_res_p {
                    *max_res_p = res;
                    *best_action.lock().unwrap() = *action;
                }
                if *max_res_p == 1 {
                    break;
                }
            }
        }));
    }
    let (tx, rx) = channel();
    let timer = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(10));
        if SystemTime::now() >= unsafe { TIME_LIMIT } {
            unsafe {
                IS_TIMEOUT = true;
            }
            break;
        }
        match rx.try_recv() {
            Ok(_) => break,
            _ => continue,
        }
    });
    for handle in handles {
        handle.join().unwrap();
    }
    tx.send(true).unwrap_or(());
    timer.join().unwrap();
    let score = *max_res.clone().lock().unwrap();
    println!("Score: {}", score);
    return *best_action.clone().lock().unwrap();
}

pub fn play(board: &Board, color: Color, _is_passed: bool) -> Move {
    let mut best_action: (i8, i8);
    unsafe {
        IS_TIMEOUT = false;
    }
    if unsafe { COUNT >= 41 && !IS_FIRST_END } {
        best_action = apply_joseki(board, color);
        if best_action.0 == -1 {
            unsafe { IS_FIRST_END = true };
            best_action = nega_scout_action(board, color, vec![DEPTH - 4, DEPTH - 1, DEPTH]);
        }
    } else if unsafe { COUNT >= 25 } {
        best_action = nega_scout_action(board, color, vec![DEPTH - 4, DEPTH - 1, DEPTH]);
    } else if unsafe { COUNT > DEPTH as u8 } {
        println!("final1");
        let scores =
            nega_scout_actions(board, color, DEPTH, &get_move_ordering_score(board, color));
        if scores.is_empty() {
            best_action = (-1, -1);
        } else if unsafe { IS_TIMEOUT } {
            best_action = scores.first().unwrap().1;
        } else {
            best_action = read_final_action(color, &scores)
        }
    } else {
        println!("final2");
        let scores = get_move_ordering_score(board, color);
        best_action = read_final_action(color, &scores);
    }
    unsafe {
        COUNT -= 2;
    }
    if best_action.0 == -1 {
        Move::Pass
    } else {
        Move::Mv(best_action.0, best_action.1)
    }
}

fn calc_move_ordering_value(board: &Board, color: Color) -> i32 {
    -(count(valid_mask(board, !color)) as i32)
}

fn nega_scout(
    board: &Board,
    board_p: &Board,
    color: Color,
    alpha: i32,
    beta: i32,
    depth: i8,
    initial_depth: i8,
    is_passed: bool,
) -> Option<i32> {
    if unsafe { IS_TIMEOUT } {
        println!("EXCEEDED");
        return None;
    }
    if depth == 0 {
        return Some(evaluate(
            board,
            board_p,
            color,
            unsafe { COUNT as i8 } + depth - initial_depth,
        ));
    }
    let valid_moves = valid_moves(board, color);
    let mut alpha = alpha;

    if valid_moves.is_empty() {
        if is_passed {
            let res = get_result(board, color);
            return Some(res as i32 * INF);
        }
        return Some(-nega_scout(
            board,
            board_p,
            !color,
            -beta,
            -alpha,
            depth - 1,
            initial_depth,
            true,
        )?);
    }

    let mut nodes = vec![];
    for action in valid_moves {
        let mut n_board = board.clone();
        do_move(&mut n_board, &Move::Mv(action.0, action.1), color);
        nodes.push((calc_move_ordering_value(&n_board, color), n_board));
    }
    nodes.sort_by(|a, b| b.0.cmp(&a.0));

    let (first, trail) = nodes.split_first().unwrap();
    let v = -nega_scout(
        &first.1,
        board,
        !color,
        -beta,
        -alpha,
        depth - 1,
        initial_depth,
        false,
    )?;
    let mut max = v;
    if beta <= v {
        return Some(v);
    }
    if alpha < v {
        alpha = v;
    }

    for (_, nboard) in trail {
        let mut score = -nega_scout(
            nboard,
            board,
            !color,
            -alpha - 1,
            -alpha,
            depth - 1,
            initial_depth,
            false,
        )?;
        if beta <= score {
            return Some(score);
        }
        if alpha < score {
            alpha = score;
            score = -nega_scout(
                nboard,
                board,
                !color,
                -beta,
                -alpha,
                depth - 1,
                initial_depth,
                false,
            )?;
            if beta <= score {
                return Some(score);
            }
            if alpha < score {
                alpha = score;
            }
        }
        if max < score {
            max = score
        }
    }
    Some(max)
}

pub fn get_move_ordering_score(board: &Board, color: Color) -> Vec<(i32, (i8, i8), Board)> {
    let valid_moves = valid_moves(board, color);
    let mut nodes = vec![];
    for action in valid_moves {
        let mut n_board = board.clone();
        do_move(&mut n_board, &Move::Mv(action.0, action.1), color);
        nodes.push((calc_move_ordering_value(&n_board, color), action, n_board));
    }
    nodes.sort_by(|a, b| b.0.cmp(&a.0));
    nodes
}

pub fn nega_scout_actions(
    board: &Board,
    color: Color,
    depth: i8,
    scores: &Vec<(i32, (i8, i8), Board)>,
) -> Vec<(i32, (i8, i8), Board)> {
    let res = Arc::new(Mutex::new(vec![]));
    let mut alpha = -INF;
    let beta = INF;
    if scores.is_empty() {
        return [].to_vec();
    }

    let (first, trail) = scores.split_first().unwrap();
    let v;
    let fst_res = nega_scout(&first.2, board, !color, -beta, -alpha, depth, depth, false);
    match fst_res {
        Some(val) => v = -val,
        None => return vec![*first],
    }

    let max = v;
    {
        res.lock().unwrap().push((v, first.1, first.2));
    }

    if alpha < v {
        alpha = v;
    }

    let alpha = Arc::new(Mutex::new(alpha));
    let max = Arc::new(Mutex::new(max));
    let len = trail.len();
    let mut handles = vec![];

    for i in 0..THREAD_NUM {
        let mut chunk = vec![];
        for j in (i..len).step_by(THREAD_NUM) {
            chunk.push(*trail.get(j).unwrap());
        }
        let alpha = alpha.clone();
        let beta = beta.clone();
        let max = max.clone();
        let res = res.clone();
        let board = board.clone();
        handles.push(thread::spawn(move || {
            for (_, action, nboard) in chunk {
                let alpha_v;
                {
                    let alpha_p = alpha.lock().unwrap();
                    alpha_v = *alpha_p;
                }
                let res_score = nega_scout(
                    &nboard,
                    &board,
                    !color,
                    -alpha_v - 1,
                    -alpha_v,
                    depth,
                    depth,
                    false,
                );

                let mut score;
                match res_score {
                    Some(val) => score = -val,
                    None => break,
                }
                if alpha_v < score {
                    let alpha_v;
                    {
                        let mut alpha_p = alpha.lock().unwrap();
                        if *alpha_p < score {
                            *alpha_p = score;
                        }
                        alpha_v = *alpha_p;
                    }
                    let res_score = nega_scout(
                        &nboard, &board, !color, -beta, -alpha_v, depth, depth, false,
                    );
                    match res_score {
                        Some(val) => score = -val,
                        None => break,
                    }
                    let mut alpha_p = alpha.lock().unwrap();
                    if *alpha_p < score {
                        *alpha_p = score;
                    }
                }
                let mut max_p = max.lock().unwrap();
                if *max_p < score {
                    *max_p = score;
                }
                res.lock().unwrap().push((score, action, nboard));
            }
        }));
    }
    let (tx, rx) = channel();
    let timer = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(10));
        if SystemTime::now() >= unsafe { TIME_LIMIT } {
            unsafe {
                IS_TIMEOUT = true;
            }
            break;
        }
        match rx.try_recv() {
            Ok(_) => break,
            _ => continue,
        }
    });
    for handle in handles {
        handle.join().unwrap();
    }
    tx.send(true).unwrap_or(());
    timer.join().unwrap();

    let mut res = res.clone().lock().unwrap().clone();

    if res.is_empty() {
        return vec![*first];
    }
    println!("Depth: {}, Score: {}", depth, max.clone().lock().unwrap());
    res.sort_by(|a, b| b.0.cmp(&a.0));
    res.to_vec()
}

fn nega_scout_action(board: &Board, color: Color, depths: Vec<i8>) -> (i8, i8) {
    let mut scores = get_move_ordering_score(board, color);

    for d in depths {
        scores = nega_scout_actions(board, color, d, &mut scores);
        if !scores.is_empty() && scores.first().unwrap().0 == INF {
            println!("will win");
            return scores.first().unwrap().1;
        }
        if unsafe { IS_TIMEOUT } {
            break;
        }
    }

    if scores.is_empty() {
        (-1, -1)
    } else {
        scores.first().unwrap().1
    }
}

/*fn apply_joseki(board: &Board, color: Color) -> (i8, i8) {
    let valid_moves = valid_moves(board, color);
    let mut best_action = (-1, -1);
    let mut max_val = 0.0 as f32; // 相手の負け確率
    if valid_moves.is_empty() {
        return best_action;
    }
    for action in valid_moves {
        let mut nboard = board.clone();
        do_move(&mut nboard, &Move::Mv(action.0, action.1), color);
        for b in expand(&nboard) {
            let res = DATABASE.get(&b);
            if !res.is_some() {
                continue;
            }
            let res = res.unwrap();
            let rate = (res.0) as f32;
            if rate > max_val {
                println!("FOUND {}", rate);
                best_action = action;
                max_val = rate;
                break;
            }
        }
    }
    best_action
}*/

use rand::prelude::*;
fn apply_joseki(board: &Board, color: Color) -> (i8, i8) {
    let valid_moves = valid_moves(board, color);
    let mut actions = vec![];
    let mut rates = vec![];
    for action in valid_moves {
        let mut nboard = board.clone();
        do_move(&mut nboard, &Move::Mv(action.0, action.1), color);
        for b in expand(&nboard) {
            let res = DATABASE.get(&b);
            if !res.is_some() {
                continue;
            }
            let res = res.unwrap();
            let rate = res.0 as u32;
            if res.0 as f32 / (res.0 + res.1) as f32 >= 0.5 {
                println!("Found: {}", rate);
                actions.push(action);
                rates.push(res.0 as u32);
            }
        }
    }
    if actions.is_empty() {
        return (-1, -1);
    }
    let rd = random::<u32>() % (rates.iter().sum::<u32>());
    println!("Rd: {}", rd);
    let mut sum = 0;
    for i in 0..rates.len() {
        sum += rates[i];
        if sum > rd {
            println!("Chosed: {}", rates[i]);
            return actions[i];
        }
    }
    *actions.last().unwrap()
}
