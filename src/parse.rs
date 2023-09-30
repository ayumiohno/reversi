use crate::command::Move;
use crate::play::{do_move, init_board};
use crate::play::{valid_moves, Board};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use once_cell::sync::Lazy;
use std::collections::HashMap;

static mut WL: Lazy<HashMap<Board, (i32, i32)>> = Lazy::new(|| HashMap::new());
static FST_STEP: usize = 20;

fn parse_line(line: &str) -> (Vec<Board>, Vec<Board>) {
    let mut p = vec![];
    let mut o = vec![];
    let terms = line.split_whitespace().collect::<Vec<&str>>();
    let cmd_term = terms.first().unwrap();
    let cmds = cmd_term.split(['+', '-']).collect::<Vec<&str>>();
    let mut board = init_board();
    for i in 1..(FST_STEP + 1) {
        let x = (cmds[i].as_bytes())[0] as i8 - 'a' as i8;
        let y = (cmds[i].as_bytes())[1] as i8 - '1' as i8;
        assert!(valid_moves(&board, i % 2 == 1).contains(&(x, y)));
        do_move(&mut board, &Move::Mv(x, y), i % 2 == 1);
        if i % 2 == 1 {
            p.push(board.clone());
        } else {
            o.push(board.clone());
        }
    }
    if terms[1].as_bytes()[0] as u8 == '+' as u8 {
        (p, o)
    } else {
        (o, p)
    }
}

pub fn create_database() {
    let file = File::open("src/logbook.gam").expect("file not found");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let (w, l) = parse_line(&line.unwrap());
        for b in w {
            unsafe {
                if WL.contains_key(&b) {
                    (*WL.get_mut(&b).unwrap()).0 += 1;
                } else {
                    WL.insert(b, (1, 0));
                }
            }
        }
        for b in l {
            unsafe {
                if WL.contains_key(&b) {
                    (*WL.get_mut(&b).unwrap()).1 += 1;
                } else {
                    WL.insert(b, (0, 1));
                }
            }
        }
    }
    let file = File::create("src/database.rs").expect("failed creating file");
    // let file = File::create("database").expect("failed creating file");
    writeln!(
        &file,
        "use crate::play::Board;
         use once_cell::sync::Lazy;
         use std::collections::HashMap;
        "
    )
    .unwrap();
    writeln!(
        &file,
        "pub static DATABASE: Lazy<HashMap<Board, (i32, i32)>> =Lazy::new(|| ["
    )
    .unwrap();
    unsafe {
        for (k, v) in WL.iter() {
            writeln!(&file, "(({}, {}), ({}, {})),", k.0, k.1, v.0, v.1).unwrap();
        }
    }
    writeln!(&file, "].iter().cloned().collect());").unwrap();
}
