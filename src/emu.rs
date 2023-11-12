use core::panic;
use std::{
    io::{Error, ErrorKind},
    str::FromStr,
};

use crate::utils::read_file_into_slice;
use chess::{Board, ChessMove, Color, Piece, Square};
use vampirc_uci::{UciFen, UciInfoAttribute, UciMessage};
use w65c02s::{System, W65C02S};
const fn calc_lcd_map() -> [char; 0x100] {
    let mut res = ['☐'; 0x100];
    let vals = [
        (' ', 0b11111111),
        ('-', 0b11110111),
        ('a', 0b10100000),
        ('b', 0b10000011),
        ('c', 0b10100111),
        ('d', 0b10100001),
        ('e', 0b10000100),
        ('f', 0b10001110),
        ('g', 0b10010000),
        ('h', 0b10001011),
        ('i', 0b11101111),
        ('j', 0b11110011),
        ('k', 0b10001010),
        ('l', 0b11001111),
        ('m', 0b11101011),
        ('n', 0b10101011),
        ('o', 0b10100011),
        ('p', 0b10001100),
        ('q', 0b10011000),
        ('r', 0b10101111),
        ('s', 0b10010010),
        ('t', 0b10000111),
        ('u', 0b11100011),
        ('v', 0b11100011),
        ('w', 0b11101011),
        ('x', 0b10001001),
        ('y', 0b10010001),
        ('z', 0b10100100),
        ('A', 0b10001000),
        ('B', 0b10000011),
        ('C', 0b11000110),
        ('D', 0b10100001),
        ('E', 0b10000110),
        ('F', 0b10001110),
        ('G', 0b11000010),
        ('H', 0b10001001),
        ('I', 0b11001111),
        ('J', 0b11100001),
        ('K', 0b10001010),
        ('L', 0b11000111),
        ('M', 0b11101010),
        ('N', 0b11001000),
        ('O', 0b11000000),
        ('P', 0b10001100),
        ('Q', 0b10010100),
        ('R', 0b11001100),
        ('S', 0b10010010),
        ('T', 0b11001110),
        ('U', 0b11000001),
        ('V', 0b11000001),
        ('W', 0b11010101),
        ('Y', 0b10010001),
        ('Z', 0b10100100),
        ('0', 0b11000000),
        ('1', 0b11111001),
        ('2', 0b10100100),
        ('3', 0b10110000),
        ('4', 0b10011001),
        ('5', 0b10010010),
        ('6', 0b10000010),
        ('7', 0b11111000),
        ('8', 0b10000000),
        ('9', 0b10010000),
        (']', 0b11110000),
        ('=', 0b11110110),
        ('K', 0b10000101),
        ('-', 0b10111111),
    ];
    let mut i = 0;
    while i < vals.len() {
        let (c, j) = vals[i as usize];
        res[j as usize] = c;
        res[(j & 0x7f) as usize] = c;
        i += 1;
    }
    res
}
const LCD_MAP: [char; 0x100] = calc_lcd_map();
const _LED_NAMES: [&str; 8] = [
    "black_led",
    "white_led",
    "calc_led",
    "mem_led",
    "pos_led",
    "play_led",
    "play_tone",
    "strobe_lcd",
];
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum MM2Button {
    CL = 0,
    POS,
    MEM,
    INFO,
    LEV,
    ENT,
    RightWhite0,
    LeftBlack9,
    E5Queen,
    F6King,
    G7,
    A1Pawn,
    H8,
    B2Knight,
    C3Bishop,
    D4Rook,
}

const PIECE_BUTTONS: [MM2Button; 6] = [
    MM2Button::A1Pawn,
    MM2Button::B2Knight,
    MM2Button::C3Bishop,
    MM2Button::D4Rook,
    MM2Button::E5Queen,
    MM2Button::F6King,
];

pub trait MephistoEmu {
    fn set_difficulty(self: &mut Self, new_difficulty: Option<u8>) -> Result<(), Error>;
    fn set_position(self: &mut Self, startpos: bool, fen: Option<UciFen>, movs: Vec<ChessMove>);
    fn set_fen(self: &mut Self, fen: &str);
    fn force_moves(self: &mut Self, movs: Vec<ChessMove>);
    fn play_move(self: &mut Self, mov: ChessMove);
    fn gen_move(self: &mut Self) -> Option<UciMessage>;
}

pub struct MM2Emu {
    cpu: W65C02S,
    pub system: MM2,
    pub cur_board: Board,
    instruction_count: u64,
    interrupt_count: u64,
    difficulty: u8,
    // key_pressed: u8,
    tone_count: u64,
    last_move_forced: bool,
}

impl MM2Emu {
    pub fn new() -> MM2Emu {
        MM2Emu {
            cpu: W65C02S::new(),
            system: MM2::new(),
            cur_board: Board::default(),
            instruction_count: 0,
            interrupt_count: 0,
            difficulty: 1,
            // key_pressed: 16,
            tone_count: 0,
            last_move_forced: false,
        }
    }
    fn await_interrupt(self: &mut MM2Emu) {
        while self.instruction_count < 2000 {
            self.cpu.step(&mut self.system);
            self.instruction_count += 1;
            if self.system.outlatch[6] {
                self.tone_count += 1;
            }
        }
        self.instruction_count = 0;
        self.interrupt_count += 1;
        self.cpu.set_irq(true);
        self.system.irq_done = false;
        while !self.system.irq_done {
            self.cpu.step(&mut self.system);
            if self.system.outlatch[6] {
                self.tone_count += 1;
            }
        }
    }
    fn wait_1sec(self: &mut MM2Emu) {
        self.interrupt_count = 0;
        for _ in 0..500 {
            self.await_interrupt();
        }
    }
    pub fn init(self: &mut MM2Emu) {
        self.cpu.reset();
        self.system.display_pos = 3;
        self.system.led_rank = 0;
        self.wait_1sec();
        self.wait_1sec();
    }
    fn press_key(self: &mut MM2Emu, button: MM2Button) {
        self.wait_1sec();
        let key_pressed = button as usize;
        self.system.pressed_keys[(key_pressed > 7) as usize][key_pressed % 8] = true;
        self.wait_1sec();
        let key_pressed = button as usize;
        self.system.pressed_keys[(key_pressed > 7) as usize][key_pressed % 8] = false;
        self.wait_1sec();
    }
    fn set_default_pos(self: &mut MM2Emu) {
        self.system.cur_bitboard = [0, 0, 0xff, 0xff, 0xff, 0xff, 0, 0];
        self.cur_board = Board::default();
        self.init();
        self.set_difficulty(None).unwrap();
    }
    fn make_half_move(self: &mut MM2Emu, sq: chess::Square) {
        self.wait_1sec();
        self.system.cur_bitboard[sq.get_rank().to_index()] ^= 1 << (sq.get_file().to_index());
        self.wait_1sec();
    }
}

impl MephistoEmu for MM2Emu {
    fn set_difficulty(self: &mut MM2Emu, new_difficulty: Option<u8>) -> Result<(), Error> {
        if let Some(diff) = new_difficulty {
            if diff < 1 || diff > 10 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Difficulty can only be from 1 to 10!",
                ));
            }
            self.difficulty = diff - 1;
        }
        const DIFFICULTIES: [MM2Button; 10] = [
            MM2Button::A1Pawn,
            MM2Button::B2Knight,
            MM2Button::C3Bishop,
            MM2Button::D4Rook,
            MM2Button::E5Queen,
            MM2Button::F6King,
            MM2Button::G7,
            MM2Button::H8,
            MM2Button::LeftBlack9,
            MM2Button::RightWhite0,
        ];
        self.press_key(MM2Button::LEV);
        self.press_key(DIFFICULTIES[(self.difficulty) as usize]);
        self.press_key(MM2Button::ENT);
        Ok(())
    }
    fn set_position(self: &mut Self, startpos: bool, fen: Option<UciFen>, movs: Vec<ChessMove>) {
        if startpos && movs.len() == 0 {
            self.set_default_pos();
            return;
        } else if startpos && movs.len() == 1 {
            self.set_default_pos();
            self.play_move(movs[0]);
            return;
        }
        let nfen: &str;
        let s: UciFen;
        let mb_last = movs.last();
        let last: ChessMove = if mb_last.is_some() {
            *mb_last.unwrap()
        } else {
            ChessMove::new(Square::A1, Square::A1, None)
        };
        if self.cur_board.legal(last) {
            let nb = self.cur_board.make_move_new(last);
            let mut ob = if startpos {
                nfen = "startpos";
                Board::default()
            } else {
                s = fen.unwrap();
                nfen = s.as_str();
                Board::from_str(nfen).unwrap()
            };
            for mov in movs.iter() {
                ob = ob.make_move_new(*mov);
            }
            if nb == ob {
                self.play_move(last);
                return;
            }
        } else if startpos {
            nfen = "startpos"
        } else {
            s = fen.unwrap();
            nfen = s.as_str();
        }
        self.set_fen(nfen);
        self.force_moves(movs);
    }
    fn set_fen(self: &mut MM2Emu, fen: &str) {
        if fen == "startpos" {
            return self.set_default_pos();
        }
        let board = match Board::from_str(fen) {
            Ok(b) => b,
            Err(e) => {
                println!(
                    "info Debug invalid fen: {fen}, Error: {e}\ninfo Debug using default Board!"
                );
                Board::default()
            }
        };
        self.system.cur_bitboard = [0; 8];
        self.cur_board = board;
        self.init();
        self.press_key(MM2Button::POS);
        self.press_key(MM2Button::ENT);
        self.wait_1sec();
        println!("info Debug cur board: {}", board);
        let mut last_piece = None;
        let mut last_color = None;
        for f in 0..8 {
            let file = chess::File::from_index(f);
            for r in 0..8 {
                let rank = chess::Rank::from_index(r);
                let sq = Square::make_square(rank, file);
                if let Some(piece) = board.piece_on(sq) {
                    let color = board.color_on(sq).unwrap();
                    if !(last_piece.is_some()
                        && last_piece.unwrap() == piece
                        && last_color.unwrap() == color)
                    {
                        self.press_key(PIECE_BUTTONS[piece.to_index()]);
                        if color == Color::Black {
                            self.press_key(PIECE_BUTTONS[piece.to_index()]);
                        }
                    }
                    last_piece = Some(piece);
                    last_color = Some(color);
                    println!(
                        "info Debug placing {} {} on {}",
                        if color == Color::White {
                            "white"
                        } else {
                            "black"
                        },
                        piece,
                        sq
                    );
                    self.make_half_move(sq);
                    self.wait_1sec();
                }
            }
        }
        self.press_key(MM2Button::CL);
        if board.side_to_move() == Color::Black {
            self.press_key(MM2Button::POS);
            self.press_key(MM2Button::LeftBlack9);
            self.press_key(MM2Button::CL);
        }
    }
    fn force_moves(self: &mut Self, movs: Vec<ChessMove>) {
        self.press_key(MM2Button::LEV);
        self.press_key(MM2Button::MEM);
        self.press_key(MM2Button::ENT);
        for mov in movs {
            self.play_move(mov);
        }
        self.last_move_forced = true;
    }
    fn play_move(self: &mut MM2Emu, mov: ChessMove) {
        if !self.cur_board.legal(mov) {
            panic!(
                "info Debug Trying invalid move! cur_board: {}",
                self.cur_board
            )
        }
        // remove piece at dest before making move
        if self.cur_board.piece_on(mov.get_dest()).is_some() {
            self.make_half_move(mov.get_dest());
        }
        self.cur_board = self.cur_board.make_move_new(mov);
        self.make_half_move(mov.get_source());
        self.make_half_move(mov.get_dest());
        self.tone_count = 0;
        // check casteling
        if mov.get_source().get_file() == chess::File::E
            && self.cur_board.piece_on(mov.get_dest()).unwrap() == chess::Piece::King
        {
            if mov.get_dest().get_file() == chess::File::G
                || mov.get_dest().get_file() == chess::File::C
            {
                let rank = mov.get_source().get_rank();
                let sec_mov;
                if mov.get_dest().get_file() == chess::File::G {
                    sec_mov = ChessMove::new(
                        chess::Square::make_square(rank, chess::File::H),
                        chess::Square::make_square(rank, chess::File::F),
                        None,
                    );
                } else {
                    sec_mov = ChessMove::new(
                        chess::Square::make_square(rank, chess::File::A),
                        chess::Square::make_square(rank, chess::File::D),
                        None,
                    );
                }
                self.make_half_move(sec_mov.get_source());
                self.make_half_move(sec_mov.get_dest());
            }
        }
        if mov.get_promotion().is_some() {
            let prom = mov.get_promotion().unwrap();
            self.press_key(PIECE_BUTTONS[prom as usize])
        }
    }
    fn gen_move(self: &mut MM2Emu) -> Option<UciMessage> {
        loop {
            self.wait_1sec();
            if self.last_move_forced || self.cur_board == Board::default() {
                self.last_move_forced = false;
                self.press_key(MM2Button::ENT);
            }
            self.wait_1sec();
            let disp_str = self
                .system
                .display
                .as_slice()
                .iter()
                .map(|a| LCD_MAP[*a as usize])
                .collect::<String>();
            while disp_str.contains('☐') {
                self.wait_1sec();
            }
            if disp_str.starts_with(" N ") {
                let num = disp_str.split_at(2 as usize).1.to_string();
                let mate_in = Some(num.trim().parse::<i8>().unwrap());
                println!(
                    "{}",
                    UciMessage::Info(vec![UciInfoAttribute::Score {
                        cp: None,
                        mate: mate_in,
                        lower_bound: None,
                        upper_bound: None
                    }])
                );
                if self.cur_board.color_on(self.system.led_square).unwrap()
                    != self.cur_board.side_to_move()
                {
                    self.make_half_move(self.system.led_square);
                }
                let start = self.system.led_square;
                self.make_half_move(start);
                while start == self.system.led_square {
                    self.wait_1sec();
                }
                let mut m = ChessMove::new(start, self.system.led_square, None);
                self.make_half_move(self.system.led_square);
                if !self.cur_board.legal(m) {
                    m = ChessMove::new(m.get_dest(), m.get_source(), None);
                }
                self.cur_board = self.cur_board.make_move_new(m);
                return Some(UciMessage::BestMove {
                    best_move: m,
                    ponder: None,
                });
            } else if disp_str.starts_with("Pr") {
                let start = self.system.led_square;
                self.make_half_move(start);
                let p_char = disp_str.chars().last().unwrap();
                let prom = match p_char {
                    'D' => Piece::Queen,
                    'T' => Piece::Rook,
                    '5' => Piece::Knight,
                    'L' => Piece::Bishop,
                    _ => panic!("Unknown Promotion"),
                };
                let m = ChessMove::new(start, self.system.led_square, Some(prom));
                self.make_half_move(self.system.led_square);
                self.press_key(PIECE_BUTTONS[prom as usize]);
                return Some(UciMessage::BestMove {
                    best_move: m,
                    ponder: None,
                });
            } else if disp_str == "PLAY" {
                self.press_key(MM2Button::ENT);
            } else if disp_str == "NAT " {
                return None;
            }
            let mov = match ChessMove::from_str(disp_str.to_lowercase().as_str()) {
                Ok(m) => m,
                Err(_) => {
                    continue;
                }
            };
            self.play_move(mov);
            self.press_key(MM2Button::INFO);
            let p_str = self
                .system
                .display
                .iter()
                .map(|a| LCD_MAP[*a as usize])
                .collect::<String>()
                .to_lowercase();
            let p_move = match ChessMove::from_str(p_str.as_str()) {
                Ok(m) => Some(m),
                Err(_) => {
                    println!("info Debug failed to parse ponder {p_str}!");
                    None
                }
            };
            self.press_key(MM2Button::A1Pawn);
            let mut info = self
                .system
                .display
                .map(|a| {
                    format!(
                        "{}{}",
                        LCD_MAP[a as usize],
                        if a & 0x80 == 0 && a != 0xff { "." } else { "" }
                    )
                })
                .join("");
            let score = (match info.trim().parse::<f32>() {
                Ok(f) => f,
                Err(_) => 0.0,
            } * 100.0) as i32;
            //        self.press_keys(MM2Button::CL);
            //self.press_keys(MM2Button::INFO);
            self.press_key(MM2Button::C3Bishop);
            info = self
                .system
                .display
                .iter()
                .map(|a| format!("{}", LCD_MAP[*a as usize]))
                .collect::<String>();
            let vinfo = info.split(' ').collect::<Vec<&str>>();
            let ninfo = if vinfo.len() > 1 { vinfo[1] } else { "0" };
            let nodes = match ninfo.trim().parse::<u8>() {
                Ok(n) => n,
                Err(e) => {
                    println!("info Debug Could not parse: {} Error: {}", info, e);
                    0
                }
            };
            self.press_key(MM2Button::CL);
            println!(
                "{}",
                UciMessage::Info(vec![
                    UciInfoAttribute::Score {
                        cp: Some(score),
                        mate: None,
                        lower_bound: None,
                        upper_bound: None
                    },
                    UciInfoAttribute::Depth(nodes)
                ])
            );
            return Some(UciMessage::BestMove {
                best_move: mov,
                ponder: p_move,
            });
        }
    }
}

pub struct MM2 {
    pub ram: [u8; 0x1000],
    book: [u8; 0x4000],
    rom: [u8; 0x8000],
    cur_bitboard: [u8; 8],
    pressed_keys: [[bool; 8]; 2],
    outlatch: [bool; 8],
    mux: usize,
    display: [u8; 4],
    last_display: [u8; 4],
    display_pos: i8,
    board_leds: [u8; 8],
    irq_done: bool,
    led_rank: usize,
    led_square: chess::Square,
}

impl MM2 {
    pub fn new() -> MM2 {
        // initialize RAM with all 0xFFs
        let ram = [0xFF; 0x1000];
        // initialize empty ROMs
        let mut book = [0x00; 0x4000];
        let mut rom = [0x00; 0x8000];
        // Read book
        read_file_into_slice("./hg240.rom", &mut book);
        // read ROM
        read_file_into_slice("./MM2.rom", &mut rom);
        MM2 {
            ram,
            book,
            rom,
            cur_bitboard: [0; 8],
            pressed_keys: [[false; 8]; 2],
            mux: 0,
            outlatch: [false; 8],
            display: [0; 4],
            last_display: [0; 4],
            display_pos: 3,
            board_leds: [0; 8],
            irq_done: true,
            led_rank: 7,
            led_square: chess::Square::A1,
        }
    }
}

impl System for MM2 {
    fn read(&mut self, _cpu: &mut W65C02S, addr: u16) -> u8 {
        match addr {
            0..=0xfff => self.ram[addr as usize],
            0x1800..=0x1807 => {
                if self.pressed_keys[self.outlatch[7] as usize][(addr & 0xf) as usize] {
                    0x7f
                } else {
                    0xff
                }
            }
            0x2000 => self.cur_bitboard[self.mux],
            0x4000..=0x7fff => self.book[(addr - 0x4000) as usize],
            0x8000.. => self.rom[(addr - 0x8000) as usize],
            _ => {
                println!("info Debug Read unknown address {:04X}! returning FF", addr);
                0xff as u8
            }
        }
    }
    fn write(&mut self, cpu: &mut W65C02S, addr: u16, value: u8) {
        match addr {
            0..=0xfff => self.ram[addr as usize] = value,
            0x1000..=0x1007 => self.outlatch[(addr & 0xf) as usize] = (value & 0x80) > 0,
            0x2800 => {
                cpu.set_irq(false);
                self.irq_done = true;
                self.display[self.display_pos as usize] =
                    if self.outlatch[7] { value } else { !value };
                self.display_pos -= 1;
                if self.display_pos < 0 {
                    self.display_pos = 3;
                    if self
                        .display
                        .iter()
                        .zip(self.last_display.iter())
                        .any(|(a, b)| a != b)
                    {
                        self.last_display.copy_from_slice(self.display.as_slice());
                        println!(
                            "{}",
                            UciMessage::Info(vec![vampirc_uci::UciInfoAttribute::Any(
                                "Display".to_string(),
                                format!(
                                    "{:?} {:?}",
                                    self.display
                                        .iter()
                                        .map(|a| format!(
                                            "{}{}",
                                            LCD_MAP[*a as usize],
                                            if *a & 0x80 == 0 { "." } else { "" }
                                        ))
                                        .collect::<String>(),
                                    self.display.map(|a| format!("{a:08b}"))
                                ),
                            )])
                        );
                    }
                }
            }
            0x3000 => {
                self.board_leds.copy_from_slice([0 as u8; 8].as_slice());
                self.board_leds[self.mux] = value;
                if value > 0 {
                    self.led_square = unsafe {
                        chess::Square::new((self.mux * 8 + value.trailing_zeros() as usize) as u8)
                    };
                }
                self.led_rank += 1;
                if self.led_rank > 7 {
                    self.led_rank = 0;
                }
            }
            0x3800 => self.mux = (!value).trailing_zeros() as usize,
            _ => println!("info Debug Ignoring write of {value} to {addr}!"),
        }
    }
}
