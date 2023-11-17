mod emu;
mod uci;
mod utils;

use emu::{MM2Emu, MephistoEmu};
use std::{str::FromStr, sync::mpsc::TryRecvError, thread, time::Duration};
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
                    "Debug" => {}
                    _ => println!("info Debug unknown option: {name}, {}", value.unwrap()),
                },
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => emu.set_position(startpos, fen, moves),
                UciMessage::Go {
                    time_control,
                    search_control: _search_control,
                } => {
                    if let Some(mov) = emu.gen_move(&stdin_channel, time_control) {
                        println!("{}", mov);
                    }
                }
                UciMessage::UciNewGame => {}
                UciMessage::Quit => return,
                _ => println!("info Debug unhandled message: {}", message),
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("Stdin disconnected!")
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}
