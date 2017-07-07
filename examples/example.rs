extern crate ghakuf;

use ghakuf::formats::*;
use ghakuf::messages::*;
use ghakuf::reader::handler::*;
use ghakuf::reader::reader::*;
use ghakuf::writer::writer::*;
use std::path::PathBuf;

fn main() {
    // build example
    let mut writer = Writer::new();
    writer.running_status(true);
    let tempo: u32 = 60 * 1000000 / 102; //bpm:102
    writer.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::SetTempo,
        data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
    });
    writer.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    writer.push(Message::TrackChange);
    writer.push(Message::MidiEvent {
        delta_time: VLQ::new(0),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3c,
            velocity: 0x7f,
        },
    });
    writer.push(Message::MidiEvent {
        delta_time: VLQ::new(192),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x40,
            velocity: 0,
        },
    });
    writer.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    writer.write(PathBuf::from("examples/example.mid")).unwrap();

    // parse example
    let mut reader = Reader::new(
        Box::new(HogeHandler {}),
        PathBuf::from("examples/example.mid"),
    ).unwrap();
    reader.read().unwrap();
}

struct HogeHandler {}
impl Handler for HogeHandler {
    fn header(&mut self, format: Format, track: u16, time_base: u16) {
        println!("Header found!");
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        println!("Meta event found!");
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        println!("MIDI event found!");
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        println!("System exclusive event found!");
    }
    fn track_change(&mut self) {
        println!("Track change occcurs!");
    }
}
