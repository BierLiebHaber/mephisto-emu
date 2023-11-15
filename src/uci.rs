use std::io;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use vampirc_uci::*;

pub fn spawn_stdin_channel() -> Receiver<UciMessage> {
    let (tx, rx) = mpsc::channel::<UciMessage>();
    let mut rdy_once = true;
    let mut debug = false;
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let message = parse_one(&buffer);
        if message.is_unknown() {
            continue;
        }
        if debug {
            println!("info Debug recieved command: {message}")
        }
        match message.clone() {
            UciMessage::IsReady => {
                if !rdy_once {
                    println!("{}", UciMessage::ReadyOk);
                    continue;
                }
                rdy_once = false;
            }
            UciMessage::SetOption { name, value } => {
                if name == "Debug" && value.is_some() {
                    debug = value.unwrap().to_lowercase() == "true";
                }
            }
            _ => {}
        }
        if let Err(_) = tx.send(message) {
            break;
        }
    });
    rx
}
pub fn print_intro() {
    let options = vec![
        UciOptionConfig::Spin {
            name: "Difficulty".to_string(),
            default: Some(1),
            min: Some(1),
            max: Some(10),
        },
        UciOptionConfig::Check {
            name: "OwnBook".to_string(),
            default: Some(true),
        },
        UciOptionConfig::Check {
            name: "Debug".to_string(),
            default: Some(false),
        },
    ];
    println!(
        "{}\n{}\n",
        UciMessage::Id {
            name: Some("Mephisto MM2".to_string()),
            author: None
        },
        UciMessage::Id {
            name: None,
            author: Some("Ulf Rathsman, Emulator by: Lukas Nöllemeyer".to_string())
        }
    );
    for o in options {
        println!("{}", UciMessage::Option(o));
    }
    println!("{}", UciMessage::UciOk)
}
