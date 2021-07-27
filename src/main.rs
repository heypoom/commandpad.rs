extern crate midir;
extern crate midly;

#[macro_use]
extern crate text_io;

use midir::MidiInput;
use midir::MidiInputConnection;
use midir::MidiInputPort;
use midir::MidiOutputConnection;
use midir::{MidiOutput, MidiOutputPort};
use rand::Rng;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::thread::sleep;
use std::time::Duration;

use std::io::stdout;
use std::io::Write;

fn midi_input(text: &'static str) -> MidiInput {
	MidiInput::new(text).unwrap()
}

fn midi_output(text: &'static str) -> MidiOutput {
	MidiOutput::new(text).unwrap()
}

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

fn position_to_note(x: u8, y: u8) -> u8 {
	81 - (10 * y) + x
}

fn get_midi_in_ports(conn: &MidiInput) -> (MidiInputPort, MidiInputPort) {
	let in_ports = conn.ports();

	for port in in_ports.clone() {
		println!("{:?}", conn.port_name(&port).unwrap());
	}

	let get_port = move |name: &'static str| {
		in_ports
			.clone()
			.into_iter()
			.find(|p| conn.port_name(p).unwrap() == name)
			.unwrap()
	};

	let daw_conn = get_port("Launchpad X LPX DAW Out");
	let midi_out = get_port("Launchpad X LPX MIDI Out");

	(daw_conn, midi_out)
}

fn get_midi_out_ports(out: &MidiOutput) -> (MidiOutputPort, MidiOutputPort) {
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
	let (daw_port, midi_port) = get_midi_out_ports(&out);

	let daw_conn = out.connect(&daw_port, "commandpad-daw-out").unwrap();

	let midi_conn = {
		let midi_out = midi_output("CommandPad MIDI Output");

		midi_out.connect(&midi_port, "commandpad-midi-out").unwrap()
	};

	(daw_conn, midi_conn)
}

pub struct Launchpad {
	color_map: HashMap<u8, u8>,

	daw_conn: MidiOutputConnection,
	midi_conn: MidiOutputConnection,

	midi_in: Option<MidiInputConnection<()>>,
}

fn empty_pad_state() -> HashMap<u8, u8> {
	let mut hm = HashMap::new();

	for i in 11..100 {
		hm.insert(i, 0);
	}

	hm
}

impl Launchpad {
	pub fn new() -> Launchpad {
		let (daw_conn, midi_conn) = initialize_output();

		Launchpad {
			color_map: empty_pad_state(),
			daw_conn,
			midi_conn,
			midi_in: None,
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
		self.color_map.insert(position, color);

		self.send(&[0b10010000, position, color]);
	}

	pub fn paint_static_grid(&mut self, grid: Vec<Vec<u8>>) {
		let mut specs: Vec<u8> = vec![240, 0, 32, 41, 2, 12, 3];

		for (y, row) in grid.into_iter().enumerate() {
			for (x, col) in row.into_iter().enumerate() {
				let note = position_to_note(x as u8, y as u8);

				println!("({}, {}) -> {}", x, y, note);
				specs.append(&mut vec![0, note, col]);
			}
		}

		specs.push(247);

		println!("colour spec: {:?}", specs);
		self.send(&specs);
	}

	pub fn paint_rgb_grid(&mut self, grid: Vec<Vec<&Vec<u8>>>) {
		let mut specs: Vec<u8> = vec![240, 0, 32, 41, 2, 12, 3];

		for (y, row) in grid.into_iter().enumerate() {
			for (x, col) in row.into_iter().enumerate() {
				let note = position_to_note(x as u8, y as u8);
				let mut spec = vec![3, note];

				let mut colcl = col.clone();
				spec.append(&mut colcl);

				println!("({}, {}) -> {}", x, y, note);
				specs.append(&mut spec);
			}
		}

		specs.push(247);

		println!("colour spec: {:?}", specs);
		self.send(&specs);
	}

	pub fn cycle_color(&mut self, position: u8) {
		let mut color = self.color_map.get(&position).unwrap_or(&0);
		if color > &127 {
			color = &0
		}

		self.light_on(position, color + 1);
	}

	pub fn setup(&mut self) {
		// Enable Programmer Mode
		self.set_programmer_mode(true);
		println!("programmer mode enabled");
	}

	pub fn clear(&mut self, color: u8) {
		for position in 11..100 {
			self.light_on(position, color);
		}
	}
}

fn rand_u8() -> u8 {
	let mut rng = rand::thread_rng();

	rng.gen_range(80..87)
}

fn main() {
	let mut launchpad = Launchpad::new();
	launchpad.setup();

	let input = midi_input("CommandPad MIDI Input");
	let (_, midi_port) = get_midi_in_ports(&input);

	let mut rng = rand::thread_rng();

	let r = &vec![127, 0, 0];
	let w = &vec![127, 127, 127];
	let fps = 15;

	for iter in 1..(10 * fps) {
		let b = &vec![
			rng.gen_range(1..127),
			rng.gen_range(1..127),
			rng.gen_range(1..127),
		];

		launchpad.paint_rgb_grid(vec![
			vec![r, r, r, r, r, r, r, r],
			vec![r, r, r, r, r, r, r, r],
			vec![w, w, w, w, w, w, w, w],
			vec![b, b, b, b, b, b, b, b],
			vec![b, b, b, b, b, b, b, b],
			vec![w, w, w, w, w, w, w, w],
			vec![r, r, r, r, r, r, r, r],
			vec![r, r, r, r, r, r, r, r],
		]);

		sleep(Duration::from_millis(1000 / fps))
	}

	let midi_in = input
		.connect(
			&midi_port,
			"CommandPad MIDI Input",
			move |timestamp_ms, message, _extra| {
				println!("{:?}", message);

				if let &[command, position, velocity] = message {
					if command == 176 && position == 97 && velocity > 0 {
						launchpad.clear(0);
						return;
					}

					if command == 176 && position == 98 && velocity > 0 {
						for position in 11..100 {
							launchpad.light_on(position, rand_u8());
						}

						return;
					}

					if command == 144 && velocity > 0 {
						println!("note on at {}", position);

						launchpad.cycle_color(position);
					}

					if command == 176 && velocity > 0 {
						println!("ctrl note on at {}", position);

						launchpad.cycle_color(position);
					}
				}
			},
			(),
		)
		.unwrap();

	loop {
		sleep(Duration::from_millis(50));
	}
}
