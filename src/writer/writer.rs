use byteorder::{BigEndian, WriteBytesExt};
use formats::*;
use messages::*;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use writer::write_error::WriteError;

pub struct Writer {
    messages: Vec<Message>,
    format: Format,
    time_base: u16,
    running_status: bool,
}
impl Writer {
    pub fn new() -> Writer {
        Writer {
            messages: Vec::new(),
            format: Format::F1,
            time_base: 480,
            running_status: false,
        }
    }
    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }
    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }
    pub fn remove(&mut self, index: usize) -> Message {
        self.messages.remove(index)
    }
    pub fn format(&mut self, format: Format) -> &mut Writer {
        self.format = format;
        self
    }
    pub fn time_base(&mut self, time_base: u16) -> &mut Writer {
        self.time_base = time_base;
        self
    }
    pub fn running_status(&mut self, running_status: bool) -> &mut Writer {
        self.running_status = running_status;
        self
    }
    pub fn write(&self, path: PathBuf) -> Result<(), WriteError> {
        let mut file = BufWriter::new(OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?);
        file.write(Tag::Header.binary())?;
        file.write(&[0, 0, 0, 6])?;
        file.write(&self.format.binary())?;
        file.write_u16::<BigEndian>(self.track_number())?;
        file.write_u16::<BigEndian>(self.time_base)?;
        let mut track_len_filo = self.track_len_filo();
        if self.messages.len() > 0 && self.messages[0] != Message::TrackChange {
            file.write(&Message::TrackChange.binary())?;
            file.write_u32::<BigEndian>(
                track_len_filo.pop().unwrap() as u32,
            )?;
        }
        let mut pre_status_byte: Option<u8> = None;
        for message in self.messages.iter() {
            match *message {
                Message::TrackChange => {
                    file.write(&Message::TrackChange.binary())?;
                    file.write_u32::<BigEndian>(
                        track_len_filo.pop().unwrap() as u32,
                    )?;
                    pre_status_byte = None;
                }
                Message::MidiEvent {
                    delta_time,
                    ref event,
                    ..
                } => {
                    let tmp_status_byte = (*event).status_byte();
                    match pre_status_byte {
                        Some(pre_status_byte) if pre_status_byte == tmp_status_byte => {
                            let tmp_message = message.binary();
                            file.write(&tmp_message[0..delta_time.len()])?;
                            file.write(&message.binary()[delta_time.len() + 1..])?;
                        }
                        _ => {
                            file.write(&message.binary())?;
                            if self.running_status {
                                pre_status_byte = Some(tmp_status_byte);
                            }
                        }
                    };
                }
                _ => {
                    file.write(&message.binary())?;
                }
            }
        }
        Ok(file.flush()?)
    }
    fn track_len_filo(&self) -> Vec<usize> {
        // First In Last Out
        let mut tracks_len: Vec<usize> = vec![0];
        let mut pre_status_byte: Option<u8> = None;
        for message in self.messages.iter() {
            match *message {
                Message::TrackChange => {
                    tracks_len.insert(0, 0);
                    pre_status_byte = None;
                }
                Message::MidiEvent { ref event, .. } => {
                    tracks_len[0] += message.len();
                    let tmp_status_byte = (*event).status_byte();
                    match pre_status_byte {
                        Some(pre_status_byte) if pre_status_byte == tmp_status_byte => {
                            tracks_len[0] -= 1;
                        }
                        _ => {
                            if self.running_status {
                                pre_status_byte = Some(tmp_status_byte);
                            }
                        }
                    };
                }
                _ => {
                    tracks_len[0] += message.len();
                }
            }
        }
        tracks_len
    }
    fn track_number(&self) -> u16 {
        let mut number = if self.messages.len() > 0 && self.messages[0] != Message::TrackChange {
            1
        } else {
            0
        };
        for message in self.messages.iter() {
            number += match *message {
                Message::TrackChange => 1,
                _ => 0,
            };
        }
        number
    }
}
