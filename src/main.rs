mod emu;
mod uci;
mod utils;

use emu::{MM2Emu, MephistoEmu};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use std::{str::FromStr, thread};
use uci::{print_intro, spawn_stdin_channel};
use vampirc_uci::UciMessage;

pub fn main() {
    let stdin_channel = spawn_stdin_channel();
    let mut emu = MM2Emu::new();
    let mut inited = false;
    let mut set_diff = 1;
    loop {
        match stdin_channel.try_recv() {
            Ok(message) => match message {
                UciMessage::Uci => print_intro(),
                UciMessage::IsReady => {
                    if !inited {
                        emu.init();
                        emu.set_difficulty(Some(set_diff)).unwrap();
                        inited = true;
                    }
                    println!("{}", UciMessage::ReadyOk);
                }
                UciMessage::SetOption { name, value } => match name.as_str() {
                    "Difficulty" => set_diff = u8::from_str(value.unwrap().as_str()).unwrap(),
                    _ => println!("unknown option: {name}, {}", value.unwrap()),
                },
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => emu.set_position(startpos, fen, moves),
                UciMessage::Go {
                    time_control: _time_control,
                    search_control: _search_control,
                } => {
                    println!(
                        "{}",
                        UciMessage::BestMove {
                            best_move: emu.gen_move().unwrap(),
                            ponder: None
                        }
                    );
                }
                UciMessage::Quit => return,
                _ => println!("unhandled message: {}", message),
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("Stdin disconnected!")
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    // emu.set_fen("startpos");
    // emu.set_difficulty(Some(4)).unwrap();
    // //emu.force_moves(vec![chess::ChessMove::from_str("d2d4").unwrap()]);
    // let input = stdin();
    // let mut rl = String::new();
    // loop {
    //     println!("CPU move: {}", emu.gen_move().unwrap());
    //     println!("{}\nEnter Move", emu.cur_board);
    //     rl.clear();
    //     match input.read_line(&mut rl) {
    //         Ok(_) => {
    //             let in_str = rl.trim();
    //             emu.play_move(chess::ChessMove::from_str(in_str).unwrap());
    //             println!("{}", emu.cur_board);
    //         }
    //         Err(e) => {
    //             panic!("Error reading from stdin: {}", e)
    //         }
    //     }
    // }
}
