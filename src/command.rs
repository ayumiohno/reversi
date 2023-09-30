use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wl {
    Win = 0,
    Lose = 1,
    Tie = 2,
}

pub enum Move {
    Mv(i8, i8),
    Pass,
    GiveUp,
}

impl Move {
    pub fn string_of_move(&self) -> String {
        match self {
            Move::Pass => String::from("PASS"),
            Move::GiveUp => String::from("GIVEUP"),
            Move::Mv(i, j) => {
                let ci = ((*i as u8) + b'A') as char;
                let cj = ((*j as u8) + b'1') as char;
                format!("{}{}", ci, cj)
            }
        }
    }
    pub fn clone(&self) -> Move {
        match self {
            Move::Pass => Move::Pass,
            Move::GiveUp => Move::GiveUp,
            Move::Mv(i, j) => Move::Mv(i.to_owned(), j.to_owned()),
        }
    }
}

pub enum Command {
    Open(String),
    End(Wl, i32, i32, String),
    Move(Move),
    Start(Color, String, i32),
    Ack(i32),
    Bye(Vec<(String, (i32, i32, i32))>),
    Empty,
}
