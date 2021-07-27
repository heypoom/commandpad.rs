extern crate midir;
extern crate midly;

#[macro_use]
extern crate text_io;

use midir::MidiOutputConnection;
use midir::{MidiOutput, MidiOutputPort};
use midly::Header;
use midly::Timing::Metrical;
use midly::TrackEvent;
use std::thread::JoinHandle;

use std::fs;
use std::io::stdout;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use midly::{MidiMessage, Smf, TrackEventKind};

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

fn prompt_midi_port(out: &MidiOutput) -> Result<MidiOutputPort, i32> {
    let out_ports = out.ports();

    for (index, port) in out_ports.iter().enumerate() {
        let name = out.port_name(port).unwrap();
        println!("Port {}: {}", index + 1, name)
    }

    let port_index = input_usize("Choose Port: ");

    if !(1..out_ports.len() + 1).contains(&port_index) {
        println!("Port out of bounds!");
        return Err(1);
    }

    let chosen_port = out_ports.get(port_index - 1).unwrap();
    let chosen_port_name = out.port_name(chosen_port).unwrap();
    println!("Selected Port: {}", chosen_port_name);

    Ok(chosen_port.clone())
}

fn play_track(
    track: &Vec<TrackEvent>,
    header: &Header,
    conn: &mut MidiOutputConnection,
    channel: u8,
) {
    println!("Format: {:?}", header.format);
    println!("Timing: {:?}", header.timing);

    let ticks_per_beat: u16 = match header.timing {
        Metrical(n) => n.into(),
        _ => 480,
    };

    let ticks_per_beat: u64 = ticks_per_beat.into();

    println!("Ticks Per Beat: {}", ticks_per_beat);

    let mut play_note = |note: u8, duration: u64, velocity: u8| {
        let note_on_msg: u8 = 0b10010000 + channel;
        let note_off_msg: u8 = 0b10000000 + channel;

        conn.send(&[note_on_msg, note, velocity]).unwrap();

        let wait_for = 500000 * duration / ticks_per_beat;
        sleep(Duration::from_micros(wait_for));

        conn.send(&[note_off_msg, note, velocity]).unwrap();
    };

    for event in track {
        match event.kind {
            TrackEventKind::Midi { message, .. } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    let time: u32 = event.delta.into();
                    play_note(key.into(), time.into(), vel.into());

                    println!(
                        "Play: (Time: {}, Key: {}, Vel: {}, CH/TH: {})",
                        time, key, vel, channel
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    let out = MidiOutput::new("Test Output").unwrap();
    let out_port = prompt_midi_port(&out).unwrap();

    let mut threads = vec![];

    for i in 0..2 {
        let out = MidiOutput::new("OP").unwrap();
        let out_port = out_port.clone();
        let mut conn_out = out.connect(&out_port, "launchmacro-output").unwrap();
        let bytes = fs::read("./test.mid").unwrap();

        let handle = std::thread::spawn(move || {
            let smf = Smf::parse(&bytes).unwrap();
            let track = smf.tracks.get(i).unwrap();
            play_track(&track, &smf.header, &mut conn_out, i as u8);
        });

        threads.push(handle);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
