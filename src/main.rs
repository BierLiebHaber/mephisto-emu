use std::{
    fs::File,
    io::{stdin, BufReader, Read},
};

use w65c02s::*;

use chess::{Board, MoveGen};

const fn calc_lcd_map() -> [char; 0x100] {
    let mut res = ['☐'; 0x100];
    let vals = [
        (' ', 0b11111111),
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
        ('T', 0b10000111),
        ('U', 0b11000001),
        ('V', 0b11000001),
        ('W', 0b11010101),
        ('X', 0b10001001),
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
        i += 1;
    }
    res
}
const LCD_MAP: [char; 0x100] = calc_lcd_map();
const LED_NAMES: [&str; 8] = [
    "black_led",
    "white_led",
    "calc_led",
    "mem_led",
    "pos_led",
    "play_led",
    "play_tone",
    "strobe_lcd",
];

const BUTTON_MAP: [&str; 16] = [
    "CL",
    "POS",
    "MEM",
    "INFO",
    "LEV",
    "ENT",
    "Right/White/0",
    "Left/Black/9",
    "E/5/Queen",
    "F/6/King",
    "G/7",
    "A/1/Pawn",
    "H/8",
    "B/2/Knight",
    "C/3/Bishop",
    "D/4/Rook",
];

pub fn main() {
    let board = Board::default();
    let movegen = MoveGen::new_legal(&board);
    assert_eq!(movegen.len(), 20);
    let mut system = MephistoEmu::new();
    let mut cpu = W65C02S::new();
    let mut instr_count = 0;
    let mut interrupt_count = 0;
    let input = stdin();
    let mut rl = String::new();
    let mut key_pressed = 16;
    let mut tone_cnt = 0;
    while cpu.get_state() != State::Stopped {
        cpu.step(&mut system);
        instr_count += 1;
        if instr_count > 2000 {
            instr_count = 0;
            interrupt_count += 1;
            cpu.set_irq(true);
        }

        if system.outlatch[6] {
            tone_cnt += 1;
        }
        if interrupt_count > 1000 {
            interrupt_count = 0;
            if key_pressed < 16 {
                system.pressed_keys[(key_pressed > 7) as usize][(key_pressed % 7) as usize] = false;
                key_pressed = 16;
                continue;
            }
            if LCD_MAP[system.display[3] as usize] == '☐' {
                continue;
            }
            println!("Tones: {}", tone_cnt);
            tone_cnt = 0;
            println!("        HGFEDCBA");
            for (row, i) in system.board_leds.iter().zip(1..=8) {
                println!("LEDs: {} {:08b}", i, row);
            }
            println!("        HGFEDCBA");
            println!("         HGFEDCBA");
            for (row, i) in system.cur_bitboard.iter().zip(1..=8) {
                println!("Board: {} {:08b}", i, row);
            }
            println!("         HGFEDCBA");
            print!("Display: ");
            for l in system.display {
                print!("{}", LCD_MAP[l as usize]);
            }
            for l in system.display {
                print!(" {:08b}", l)
            }
            for (name, val) in LED_NAMES.iter().zip(system.outlatch) {
                if val {
                    print!(" {}", name);
                }
            }
            println!("\nselect Square to invert or enter a key (from 0 to 15)");
            rl.clear();
            match input.read_line(&mut rl) {
                Ok(_) => {
                    let in_str = rl.trim();
                    let in_bytes = in_str.as_bytes();
                    if in_str.len() != 2
                        || in_bytes[0] < 'a' as u8
                        || in_bytes[0] > 'h' as u8
                        || in_bytes[1] < '1' as u8
                        || in_bytes[1] > '8' as u8
                    {
                        println!("invalid square: {}, trying as key", rl);
                        let mut key: u8 = 16;
                        for (i, but) in (0..16).zip(BUTTON_MAP) {
                            if but
                                .split('/')
                                .any(|a| a == in_str || a.to_lowercase() == in_str)
                            {
                                key = i;
                                break;
                            }
                        }
                        if key > 15 {
                            println!("invalid key: {}", key);
                        } else {
                            system.pressed_keys[(key > 7) as usize][(key % 7) as usize] = true;
                            key_pressed = key;
                        }
                    } else {
                        let rank = (in_bytes[1] - '1' as u8) as usize;
                        let file = (in_bytes[0] - 'a' as u8) as u8;
                        system.cur_bitboard[rank] = system.cur_bitboard[rank] ^ (1 << file);
                    }
                }
                Err(e) => {
                    panic!("Error reading from stdin: {}", e)
                }
            }
        }
    }
}

struct MephistoEmu {
    ram: [u8; 0x1000],
    book: [u8; 0x4000],
    rom: [u8; 0x8000],
    cur_bitboard: [u8; 8],
    pressed_keys: [[bool; 8]; 2],
    outlatch: [bool; 8],
    mux: u8,
    display: [u8; 4],
    last_display: [u8; 4],
    display_pos: i8,
    board_leds: [u8; 8],
}

impl MephistoEmu {
    pub fn new() -> MephistoEmu {
        // initialize RAM with all 0xFFs
        let ram = [0xFF; 0x1000];
        // initialize empty ROMs
        let mut book = [0x00; 0x4000];
        let mut rom = [0x00; 0x8000];
        // Read book
        let mut reader;
        let mut buffer;
        match File::open("./hg240.rom") {
            Ok(f) => {
                reader = BufReader::new(f);
                buffer = Vec::new();
                match reader.read_to_end(&mut buffer) {
                    Ok(_) => {
                        book.copy_from_slice(buffer.as_slice());
                    }
                    Err(e) => {
                        println!("Could not read book file! Error: {}", e)
                    }
                };
            }
            Err(e) => {
                println!("Could not open book file! Error: {}", e)
            }
        };

        // read ROM
        match File::open("./MM2.rom") {
            Ok(f) => {
                reader = BufReader::new(f);
                buffer = Vec::new();
                match reader.read_to_end(&mut buffer) {
                    Ok(_) => {
                        rom.copy_from_slice(buffer.as_slice());
                    }
                    Err(e) => {
                        panic!("Could not read ROM file! Error: {}", e)
                    }
                };
            }
            Err(e) => {
                panic!("Could not open ROM file! Error: {}", e)
            }
        };
        // setup bitboard
        let cur_bitboard = [0, 0, 0xff, 0xff, 0xff, 0xff, 0, 0];
        let pressed_keys = [[false; 8]; 2];
        let mux = 0;
        let outlatch = [false; 8];
        let display = [0; 4];
        let last_display = [0; 4];
        let display_pos = 3;
        let board_leds = [0; 8];
        MephistoEmu {
            ram,
            book,
            rom,
            cur_bitboard,
            pressed_keys,
            mux,
            outlatch,
            display,
            last_display,
            display_pos,
            board_leds,
        }
    }
}

impl System for MephistoEmu {
    fn read(&mut self, _cpu: &mut W65C02S, addr: u16) -> u8 {
        fn print_unknown(addr: u16) -> u8 {
            println!("Read unknown address {:04X}! returning FF", addr);
            0xff as u8
        }
        if addr < 0x1000 {
            self.ram[addr as usize]
        } else if addr < 0x1800 {
            print_unknown(addr)
        } else if addr < 0x1808 {
            if self.pressed_keys[self.outlatch[7] as usize][(addr & 0xf) as usize] {
                0x7f
            } else {
                0xff
            }
        } else if addr < 0x2000 {
            print_unknown(addr)
        } else if addr == 0x2000 {
            self.cur_bitboard[self.mux as usize]
        } else if addr < 0x4000 {
            print_unknown(addr)
        } else if addr < 0x8000 {
            self.book[(addr - 0x4000) as usize]
        } else {
            self.rom[(addr - 0x8000) as usize]
        }
    }
    fn write(&mut self, _cpu: &mut W65C02S, addr: u16, value: u8) {
        if addr < 0x1000 {
            self.ram[addr as usize] = value;
        } else if addr < 0x1008 {
            self.outlatch[(addr & 0xf) as usize] = (value & 0x80) > 0;
        } else if addr == 0x2800 {
            _cpu.set_irq(false);
            self.display[self.display_pos as usize] = if self.outlatch[7] { value } else { !value };
            self.display_pos -= 1;
            if self.display_pos < 0 {
                self.display_pos &= 3;
                if self
                    .display
                    .iter()
                    .zip(self.last_display.iter())
                    .any(|(a, b)| a != b)
                {
                    self.last_display.copy_from_slice(self.display.as_slice());
                    for l in self.display {
                        print!("{}", LCD_MAP[l as usize]);
                    }
                    println!();
                }
            }
        } else if addr == 0x3000 {
            self.board_leds[self.mux as usize] = value;
        } else if addr == 0x3800 {
            self.mux = (!value).trailing_zeros() as u8;
        } else {
            println!("Writing {:02X} to {:04X}", value, addr);
        }
    }
}
