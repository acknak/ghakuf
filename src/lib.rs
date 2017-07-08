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
//! use ghakuf::formats::*;
//! use ghakuf::messages::*;
//! use ghakuf::reader::handler::*;
//! use ghakuf::reader::reader::*;
//! use std::path::PathBuf;
//!
//! let _ = Reader::new(
//!     Box::new(HogeHandler {}),
//!     PathBuf::from("tests/test.mid"),
//! );
//!
//! struct HogeHandler {}
//! impl Handler for HogeHandler {
//!     fn header(&mut self, format: Format, track: u16, time_base: u16) {
//!       // Something
//!     }
//!     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
//!       // you
//!     }
//!     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
//!       // want
//!     }
//!     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
//!       // to
//!     }
//!     fn track_change(&mut self) {
//!       // do
//!     }
//! }
//! ```
//!
//! ## Builder
//!
//! `ghakuf` build SMF by Message enums. Message enum consists of MetaEvent, MidiEvent, SysExEvent, and TrackChange. You can use running status if you want. At track change, you should use not only MetaEvent::EndOfTrack message, but also TrackChange message.
//!
//! ```
//! use ghakuf::formats::*;
//! use ghakuf::messages::*;
//! use ghakuf::writer::writer::*;
//! use std::path::PathBuf;
//!
//! let mut writer = Writer::new();
//! writer.running_status(true);
//! let tempo: u32 = 60 * 1000000 / 102; //bpm:102
//! writer.push(Message::MetaEvent {
//!     delta_time: VLQ::new(0),
//!     event: MetaEvent::SetTempo,
//!     data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
//! });
//! writer.push(Message::MetaEvent {
//!     delta_time: VLQ::new(0),
//!     event: MetaEvent::EndOfTrack,
//!     data: Vec::new(),
//! });
//! writer.push(Message::TrackChange);
//! writer.push(Message::MidiEvent {
//!     delta_time: VLQ::new(0),
//!     event: MidiEvent::NoteOn { ch: 0, note: 0x3c, velocity: 0x7f },
//! });
//! writer.push(Message::MidiEvent {
//!     delta_time: VLQ::new(192),
//!     event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
//! });
//! writer.push(Message::MetaEvent {
//!     delta_time: VLQ::new(0),
//!     event: MetaEvent::EndOfTrack,
//!     data: Vec::new(),
//! });
//! writer.write(PathBuf::from("tests/test.mid"));
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
