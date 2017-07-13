//! A Rust library for parsing/building SMF (Standard MIDI File).
//!
//! # Examples
//!
//! `ghakuf` has parse module and build module separatory.
//!
//! ## Perser
//!
//! `ghakuf`'s parser is made by Event-driven online algorithm. You must prepare original handler implementing Handler trait to catch SMF messages. Any number of handlers you can add for parser if you want.
//!
//! ```
//! use ghakuf::messages::*;
//! use ghakuf::reader::*;
//!
//! let mut reader = Reader::new(
//!     Box::new(HogeHandler {}),
//!     "tests/test.mid",
//! ).unwrap();
//! let _ = reader.read();
//!
//! struct HogeHandler {}
//! impl Handler for HogeHandler {
//!     fn header(&mut self, format: u16, track: u16, time_base: u16) {
//!       let _ = (format, track, time_base);
//!     }
//!     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
//!       let _ = (delta_time, event, data);
//!     }
//!     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
//!       let _ = (delta_time, event);
//!     }
//!     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
//!       let _ = (delta_time, event, data);
//!     }
//!     fn track_change(&mut self) {}
//! }
//! ```
//!
//! ## Builder
//!
//! `ghakuf` build SMF by Message enums. Message enum consists of MetaEvent, MidiEvent, SysExEvent, and TrackChange. You can use running status if you want. At track change, you should use not only MetaEvent::EndOfTrack message, but also TrackChange message.
//!
//! ```
//! use ghakuf::messages::*;
//! use ghakuf::writer::*;
//!
//! let mut writer = Writer::new();
//! writer.running_status(true);
//! let tempo: u32 = 60 * 1000000 / 102; //bpm:102
//! writer.push(Message::MetaEvent {
//!     delta_time: 0,
//!     event: MetaEvent::SetTempo,
//!     data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
//! });
//! writer.push(Message::MetaEvent {
//!     delta_time: 0,
//!     event: MetaEvent::EndOfTrack,
//!     data: Vec::new(),
//! });
//! writer.push(Message::TrackChange);
//! writer.push(Message::MidiEvent {
//!     delta_time: 0,
//!     event: MidiEvent::NoteOn { ch: 0, note: 0x3c, velocity: 0x7f },
//! });
//! writer.push(Message::MidiEvent {
//!     delta_time: 192,
//!     event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
//! });
//! writer.push(Message::MetaEvent {
//!     delta_time: 0,
//!     event: MetaEvent::EndOfTrack,
//!     data: Vec::new(),
//! });
//! writer.write("tests/lib_doctest.mid");
//! ```
//!
extern crate byteorder;
#[macro_use]
extern crate log;

/// SMF Formats and Variable Length Quantity
pub mod formats;
/// enums representing SMF messages (Meta event, MIDI event, System exclusive event)
pub mod messages;
/// SMF parser and handler
pub mod reader;
/// SMF builder
pub mod writer;
