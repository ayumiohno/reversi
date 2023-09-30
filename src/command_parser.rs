use crate::command::Command;
use crate::command::Move;
use crate::command::Wl;
use crate::Color;

fn color_from_string(str: &str) -> Color {
    match str {
        "BLACK" => true,
        "WHITE" => false,
        _ => panic!("Invalid Color : {}.", str),
    }
}

fn wl_from_string(str: &str) -> Wl {
    match str {
        "WIN" => Wl::Win,
        "LOSE" => Wl::Lose,
        "TIE" => Wl::Tie,
        _ => panic!("Invalid Color : {}.", str),
    }
}

#[derive(Debug, Clone)]
struct PlaceError;

fn move_from_string(str: &str) -> Move {
    match str {
        "PASS" => Move::Pass,
        "GIVEUP" => Move::GiveUp,
        str => Move::Mv(
            str.as_bytes()[0] as i8 - ('A' as i8),
            str.as_bytes()[1] as i8 - ('1' as i8),
        ),
    }
}

pub fn parse_command(line: &String) -> Command {
    let mut tokens = line.split_whitespace();
    match tokens.next() {
        Some("MOVE") => {
            if let Some(place) = tokens.next() {
                return Command::Move(move_from_string(place));
            }
        }
        Some("START") => {
            if let (Some(wb), Some(opponent_name), Some(time)) =
                (tokens.next(), tokens.next(), tokens.next())
            {
                if let Ok(time) = time.parse() {
                    return Command::Start(color_from_string(wb), opponent_name.to_string(), time);
                }
            }
        }
        Some("ACK") => {
            if let Some(time) = tokens.next() {
                if let Ok(time) = time.parse() {
                    return Command::Ack(time);
                }
            }
        }
        Some("END") => {
            if let (Some(wl), Some(n), Some(m), Some(reason)) =
                (tokens.next(), tokens.next(), tokens.next(), tokens.next())
            {
                if let (Ok(n), Ok(m)) = (n.parse(), m.parse()) {
                    return Command::End(wl_from_string(wl), n, m, reason.to_string());
                }
            }
        }
        Some("BYE") => {
            let mut stat = Vec::new();
            while let (Some(player), Some(score), Some(wins), Some(loses)) =
                (tokens.next(), tokens.next(), tokens.next(), tokens.next())
            {
                if let (Ok(score), Ok(wins), Ok(loses)) =
                    (score.parse(), wins.parse(), loses.parse())
                {
                    stat.push((player.to_string(), (score, wins, loses)));
                }
            }
            return Command::Bye(stat);
        }
        _ => {}
    }

    Command::Empty
}
