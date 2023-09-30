use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
mod ai;
mod color;
mod command;
mod command_parser;
mod database;
mod parse;
mod play;
use crate::ai::*;
use crate::color::Color;
use crate::command::Command;
use crate::command::Move;
use crate::command::Wl;
use crate::command_parser::parse_command;
use crate::parse::*;
use crate::play::*;
use getopts::Options;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};

use once_cell::sync::Lazy;
use std::sync::RwLock;

static OPT_HOST: Lazy<RwLock<String>> = Lazy::new(|| "localhost".to_string().into());
static OPT_PORT: Lazy<RwLock<u16>> = Lazy::new(|| 3000.into());
static OPT_PLAYER_NAME: Lazy<RwLock<String>> = Lazy::new(|| "Anon,".to_string().into());
static OPT_VERBOSE: Lazy<RwLock<bool>> = Lazy::new(|| false.into());

static mut PARSE_MODE: bool = false;

fn parameters() {
    let mut opts = Options::new();
    opts.optopt("H", "host", "host name (default = localhost)", "NAME");
    opts.optopt("p", "port", "port number (default = 3000)", "NUMBER");
    opts.optopt("n", "player_name", "player name (default = Anon.)", "NAME");
    opts.optflagopt("v", "verbose", "verbose mode", "BOOL");
    opts.optflagopt("P", "parse", "database parse mode", "BOOL");

    let args: Vec<String> = std::env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("H") {
        *OPT_HOST.try_write().unwrap() = matches.opt_str("H").unwrap().to_owned();
    }
    if matches.opt_present("p") {
        *OPT_PORT.try_write().unwrap() = matches.opt_str("p").unwrap().to_owned().parse().unwrap();
    };
    if matches.opt_present("n") {
        *OPT_PLAYER_NAME.try_write().unwrap() = matches.opt_str("n").unwrap().to_owned();
    };
    if matches.opt_present("v") {
        *OPT_VERBOSE.try_write().unwrap() = true;
    }
    if matches.opt_present("P") {
        unsafe { PARSE_MODE = true };
    }
}

enum OpMove {
    PMove(Move),
    OMove(Move),
}

impl OpMove {
    fn string_of_opmove(&self) -> String {
        match self {
            OpMove::PMove(mv) => format!("+{}", mv.string_of_move()),
            OpMove::OMove(mv) => format!("-{}", mv.string_of_move()),
        }
    }
}

type Hist = Vec<OpMove>;

fn string_of_hist(hist: &Hist) -> String {
    hist.iter()
        .map(|opmove| opmove.string_of_opmove())
        .collect::<Vec<String>>()
        .join("")
}

fn print_hist(hist: &Hist) {
    println!("{}", string_of_hist(hist));
}

type Scores = Vec<(String, (i32, i32, i32))>;
fn string_of_scores(scores: &Scores) -> String {
    let maxlen = scores.iter().fold(0, |maxlen, (a, _)| a.len().max(maxlen));

    let maxslen = scores.iter().fold(0, |maxslen, (_, (s, _, _))| {
        s.to_string().len().max(maxslen)
    });

    scores
        .iter()
        .map(|(a, (s, w, l))| {
            format!(
                "{}:{}{}{} (Win {}, Lose {})\n",
                a,
                " ".repeat(maxlen + 1 - a.len()),
                " ".repeat(maxslen - s.to_string().len()),
                s,
                w,
                l
            )
        })
        .collect::<String>()
}

fn print_scores(scores: &Scores) {
    println!("{}", string_of_scores(scores));
}

fn input_command(ic: &mut BufReader<TcpStream>, stream: &TcpStream) -> Command {
    ic.chain(stream);
    let mut line = String::new();
    ic.read_line(&mut line).unwrap();
    println!("Received: {}", line);
    parse_command(&line)
}

fn wait_start(ic: &mut BufReader<TcpStream>, stream: &TcpStream) {
    loop {
        let command = input_command(ic, stream);
        match command {
            Command::Bye(scores) => {
                print_scores(&scores);
                break;
            }
            Command::Start(color, oname, time) => {
                let mut board = init_board();
                set_time_remain(time as u64);
                init_ai(color);
                if color {
                    my_move(ic, stream, &mut board, color, &mut vec![], &oname, false);
                } else {
                    op_move(ic, stream, &mut board, color, &mut vec![], &oname);
                }
                break;
            }
            _ => panic!("Invalid Command"),
        }
    }
}

fn output_command(stream: &TcpStream, command: &Command) {
    let mut tcp_writer = BufWriter::new(stream);
    match command {
        Command::Move(mv) => {
            println!("send: MOVE {}", mv.string_of_move());
            writeln!(tcp_writer, "MOVE {}", mv.string_of_move()).unwrap();
        }
        Command::Open(s) => {
            writeln!(tcp_writer, "OPEN {}", s).unwrap();
        }
        _ => panic!("Invalid Command"),
    }
}

fn my_move(
    ic: &mut BufReader<TcpStream>,
    stream: &TcpStream,
    board: &mut Board,
    color: Color,
    hist: &mut Hist,
    oname: &str,
    is_passed: bool,
) {
    let pmove = play(board, color, is_passed);
    {
        let _ = output_command(stream, &Command::Move(pmove.clone()));
    }
    if *OPT_VERBOSE.try_read().unwrap() {
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!("PMove: {} {:?}", pmove.string_of_move(), color);
        print_board(board);
    }

    do_move(board, &pmove, color);
    let command = input_command(ic, stream);
    match command {
        Command::Ack(mytime) => {
            hist.push(OpMove::PMove(pmove));
            set_time_remain(mytime as u64);
            op_move(ic, stream, board, color, hist, oname);
        }
        Command::End(wl, n, m, r) => proc_end(ic, stream, board, color, hist, oname, wl, n, m, &r),
        _ => panic!("Invalid Command"),
    }
}

fn op_move(
    ic: &mut BufReader<TcpStream>,
    stream: &TcpStream,
    board: &mut Board,
    color: Color,
    hist: &mut Hist,
    oname: &str,
) {
    let command = input_command(ic, stream);
    match command {
        Command::Move(omove) => {
            do_move(board, &omove, !color);
            let is_passed;
            match omove {
                Move::Pass => is_passed = true,
                _ => is_passed = false,
            }
            hist.push(OpMove::OMove(omove));
            my_move(ic, stream, board, color, hist, oname, is_passed)
        }
        Command::End(wl, n, m, r) => proc_end(ic, stream, board, color, hist, oname, wl, n, m, &r),
        _ => panic!("Invalid Command"),
    }
}

fn proc_end(
    ic: &mut BufReader<TcpStream>,
    stream: &TcpStream,
    board: &mut Board,
    color: Color,
    hist: &mut Hist,
    oname: &str,
    wl: Wl,
    n: i32,
    m: i32,
    r: &str,
) {
    match wl {
        Wl::Win => println!("You win! ({} vs. {}) -- {}.", n, m, r),
        Wl::Lose => println!("You lose! ({} vs. {}) -- {}.", n, m, r),
        Wl::Tie => println!("Draw ({} vs. {}) -- {}.", n, m, r),
    }
    println!(
        "Your name: {} ({})  Opponentname: {} ({}).",
        OPT_PLAYER_NAME.try_read().unwrap(),
        color,
        oname,
        !color
    );
    print_board(board);
    print_hist(hist);

    wait_start(ic, stream);
}

fn client(host: &str, port: u16) {
    let addr = format!("{}:{}", host, port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    println!("Connecting to {} {}.", host, port);
    let stream = TcpStream::connect(addr).unwrap();
    println!("Connection Ok.");
    output_command(
        &stream,
        &Command::Open(OPT_PLAYER_NAME.try_read().unwrap().to_string()),
    );
    let mut ic = BufReader::new(stream.try_clone().unwrap());
    wait_start(&mut ic, &stream);
}

fn main() {
    parameters();
    if unsafe { PARSE_MODE } {
        create_database();
        return;
    }
    let (host, port) = (OPT_HOST.try_read().unwrap(), OPT_PORT.try_read().unwrap());
    client(&host, *port);
}
