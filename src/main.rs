extern crate midir;
extern crate midly;

#[macro_use]
extern crate text_io;

use midir::MidiOutputConnection;
use midir::{MidiOutput, MidiOutputPort};

use std::io::stdout;
use std::io::Write;

fn print(s: &str) {
	print!("{}", s);
	stdout().flush().unwrap();
}

fn input(prompt: &str) -> String {
	print(prompt);
	read!("{}\n")
}

fn input_usize(prompt: &str) -> usize {
	print(prompt);
	read!("{}\n")
}

fn get_midi_ports(out: &MidiOutput) -> (MidiOutputPort, MidiOutputPort) {
	let out_ports = out.ports();

	let get_port = move |name: &'static str| {
		out_ports
			.clone()
			.into_iter()
			.find(|p| out.port_name(p).unwrap() == name)
			.unwrap()
	};

	let daw_conn = get_port("Launchpad X LPX DAW In");
	let midi_out = get_port("Launchpad X LPX MIDI In");

	(daw_conn, midi_out)
}

pub fn initialize_output() -> (MidiOutputConnection, MidiOutputConnection) {
	let out = midi_output("CommandPad DAW Output");
	let (daw_port, midi_port) = get_midi_ports(&out);

	let daw_conn = out.connect(&daw_port, "commandpad-daw-out").unwrap();

	let midi_conn = {
		let midi_out = midi_output("CommandPad MIDI Output");

		midi_out.connect(&midi_port, "commandpad-midi-out").unwrap()
	};

	(daw_conn, midi_conn)
}

pub struct Launchpad {
	daw_conn: MidiOutputConnection,
	midi_conn: MidiOutputConnection,
}

impl Launchpad {
	pub fn new() -> Launchpad {
		let (daw_conn, midi_conn) = initialize_output();

		Launchpad {
			daw_conn,
			midi_conn,
		}
	}

	pub fn send(&mut self, message: &[u8]) {
		self.midi_conn.send(message).unwrap();
	}

	pub fn send_daw(&mut self, message: &[u8]) {
		self.daw_conn.send(message).unwrap();
	}

	pub fn set_programmer_mode(&mut self, is_enabled: bool) {
		let mode = if is_enabled { 1 } else { 0 };

		self.send(&[240, 0, 32, 41, 2, 12, 14, mode, 247]);
	}

	pub fn light_on(&mut self, position: u8, color: u8) {
		self.send(&[0b10010000, position, color]);
	}

	pub fn setup(&mut self) {
		// Enable Programmer Mode
		self.set_programmer_mode(true);
		println!("programmer mode enabled");
	}
}

fn midi_output(text: &'static str) -> MidiOutput {
	MidiOutput::new(text).unwrap()
}

fn main() {
	let mut launchpad = Launchpad::new();
	launchpad.setup();

	for position in 10..100 {
		launchpad.light_on(position, 87);
	}
}
