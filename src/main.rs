mod launchpad;

#[macro_use]
extern crate text_io;
extern crate midir;

use crate::launchpad::{blank_rgb_canvas, get_midi_in_ports, midi_input, rand_u8, Launchpad};

use std::io::{prelude::*, Result};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use rand::Rng;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut launchpad = Launchpad::new();
    launchpad.setup();

    let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();

    let input = midi_input("CommandPad MIDI Input");
    let (_, midi_port) = get_midi_in_ports(&input);

    let midi_in = input
        .connect(
            &midi_port,
            "CommandPad MIDI Input",
            move |ms, message, _extra| {
                if let &[command, position, velocity] = message {
                    if command == 144 && velocity > 0 {
                        tx.send(100).unwrap();
                    }
                }
            },
            (),
        )
        .unwrap();

    println!("root 2");

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    listener.set_ttl(100)?;

    launchpad.clear(0);

    for stream in listener.incoming() {
        handle_connection(stream?, &mut launchpad);
    }

    Ok(())
}

enum Instruction {
    NOOP,

    SetTileColor,
    SetTileRgb,

    SetGridRaw,
    SetGridColor,
    SetGridRgb,
}

fn get_instruction(id: u8) -> Instruction {
    match id {
        0x03 => Instruction::SetTileColor,
        0x04 => Instruction::SetTileRgb,
        0x05 => Instruction::SetGridRaw,
        0x06 => Instruction::SetGridColor,
        0x07 => Instruction::SetGridRgb,
        _ => Instruction::NOOP,
    }
}

// |   id | action         | parameters                   |
// |------+----------------+------------------------------|
// | 0x03 | SET_TILE_COLOR | (position, swatch)           |
// | 0x04 | SET_TILE_RGB   | (position, red, green, blue) |
// | 0x05 | SET_GRID_RAW   |
// | 0x06 | SET_GRID_COLOR | (...swatch)                  |
// | 0x07 | SET_GRID_RGB   | *(red, green, blue)          |

fn handle_connection(mut stream: TcpStream, launchpad: &mut Launchpad) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let [instruction_id, payload @ ..] = buffer;

    match get_instruction(instruction_id) {
        Instruction::SetTileColor => {
            let [position, color, ..] = payload;
            println!("set tile color at {} to {}", position, color);

            launchpad.light_on(position, color);
        }

        Instruction::SetTileRgb => {
            let [position, r, g, b, ..] = payload;
            println!("set tile at {} as rgb({}, {}, {})", position, r, g, b);

            launchpad.rgb(position, [r, g, b]);
        }

        Instruction::SetGridRaw => launchpad.light_grid(payload.into()),
        Instruction::SetGridColor => launchpad.light_grid(payload.into()),
        Instruction::SetGridRgb => launchpad.light_grid(payload.into()),

        Instruction::NOOP => {}
    }
}
