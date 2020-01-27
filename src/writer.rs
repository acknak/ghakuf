use byteorder::{BigEndian, WriteBytesExt};
use formats::*;
use messages::*;
use std::io::Write;
use std::{fs, io, path};

/// `ghakuf`'s SMF builder.
///
/// This builds SMF by Message enums and write out. Message enum consists of MetaEvent, MidiEvent, SysExEvent, and TrackChange. You can use running status if you want. At track change, you should use not only MetaEvent::EndOfTrack message, but also TrackChange message.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::*;
/// use ghakuf::writer::*;
/// use std::path;
///
/// let path = path::Path::new("tests/writer_doctest.mid");
/// let tempo: u32 = 60 * 1000000 / 102; //bpm:102
/// let mut messages: Vec<Message> = vec![
///     Message::MetaEvent {
///         delta_time: 0,
///         event: MetaEvent::SetTempo,
///         data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
///     },
///     Message::MetaEvent {
///         delta_time: 0,
///         event: MetaEvent::EndOfTrack,
///         data: Vec::new(),
///     },
///     Message::TrackChange,
///     Message::MidiEvent {
///         delta_time: 0,
///         event: MidiEvent::NoteOn { ch: 0, note: 0x3c, velocity: 0x7f },
///     },
///     Message::MidiEvent {
///         delta_time: 192,
///         event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
///     },
///     Message::MetaEvent {
///         delta_time: 0,
///         event: MetaEvent::EndOfTrack,
///         data: Vec::new(),
///     }
/// ];
///
/// let mut writer = Writer::new();
/// writer.running_status(true);
/// for message in &messages {
///     writer.push(&message);
/// }
/// writer.write(&path);
/// ```
pub struct Writer<'a> {
    messages: Vec<&'a Message>,
    format: Format,
    time_base: u16,
    running_status: bool,
}
impl<'a> Writer<'a> {
    /// Builds Writer with initial value.
    ///
    /// | Writer's member | type | initial value |
    /// |:---|:---|:---|
    /// | messages | Vec\<&'a ghakuf::messages::Message\> | Vec::new() |
    /// | format | ghakuf::formats::Format | ghakuf::formats::Format::F1 |
    /// | time_base | u16 | 480 |
    /// | running_status | bool | false |
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::writer::Writer;
    ///
    /// let writer: Writer = Writer::new();
    /// ```
    pub fn new() -> Writer<'a> {
        Writer {
            messages: Vec::new(),
            format: Format::F1,
            time_base: 480,
            running_status: false,
        }
    }
    /// Returns keeping messages by borrowing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::Message;
    /// use ghakuf::writer::Writer;
    ///
    /// let message = Message::TrackChange;
    /// let mut writer: Writer = Writer::new();
    /// writer.push(&message);
    /// assert_eq!(*writer.messages(), vec![&message]);
    /// ```
    pub fn messages(&self) -> &Vec<&'a Message> {
        &self.messages
    }
    /// Pushes message to writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{Message, MidiEvent};
    /// use ghakuf::writer::Writer;
    ///
    /// let message = Message::MidiEvent {
    ///     delta_time: 0,
    ///     event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
    /// };
    /// let mut writer: Writer = Writer::new();
    /// writer.push(&message);
    /// ```
    pub fn push(&mut self, message: &'a Message) {
        self.messages.push(message);
    }
    /// Removes message from writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{Message, MetaEvent, MidiEvent};
    /// use ghakuf::writer::Writer;
    ///
    /// let mut messages_a: Vec<Message> = vec![
    ///     Message::MidiEvent {
    ///         delta_time: 0,
    ///         event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
    ///     },
    ///     Message::TrackChange,
    ///     Message::MetaEvent {
    ///         delta_time: 0,
    ///         event: MetaEvent::EndOfTrack,
    ///         data: Vec::new(),
    ///     }
    /// ];
    /// let mut writer: Writer = Writer::new();
    /// for message in &messages_a {
    ///     writer.push(&message);
    /// }
    /// writer.remove(1);
    /// assert_eq!(*writer.messages()[0], Message::MidiEvent {
    ///     delta_time: 0,
    ///     event: MidiEvent::NoteOn { ch: 0, note: 0x40, velocity: 0 },
    /// });
    /// assert_eq!(*writer.messages()[1], Message::MetaEvent {
    ///     delta_time: 0,
    ///     event: MetaEvent::EndOfTrack,
    ///     data: Vec::new(),
    /// });
    /// ```
    pub fn remove(&mut self, index: usize) -> &'a Message {
        self.messages.remove(index)
    }
    /// Sets SMF format value (Format 0, Format 1 or Format 2) by formats::Format::*.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::writer::Writer;
    ///
    /// let mut writer: Writer = Writer::new();
    /// writer.format(0);
    /// ```
    pub fn format(&mut self, format: u16) -> &mut Writer<'a> {
        self.format = Format::new(format);
        self
    }
    /// Sets SMF time base value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::writer::Writer;
    ///
    /// let mut writer: Writer = Writer::new();
    /// writer.time_base(960);
    /// ```
    pub fn time_base(&mut self, time_base: u16) -> &mut Writer<'a> {
        self.time_base = time_base;
        self
    }
    /// Sets bool value whether you adopt running statusor not.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::writer::Writer;
    ///
    /// let mut writer: Writer = Writer::new();
    /// writer.running_status(true);
    /// ```
    pub fn running_status(&mut self, running_status: bool) -> &mut Writer<'a> {
        self.running_status = running_status;
        self
    }
    /// Writes out SMF messages you stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::*;
    /// use ghakuf::writer::*;
    /// use std::path;
    ///
    /// let path = path::Path::new("tests/writer_write_doctest.mid");
    /// let tempo: u32 = 60 * 1000000 / 102; //bpm:102
    /// let mut messages: Vec<Message> = vec![
    ///     Message::MetaEvent {
    ///         delta_time: 0,
    ///         event: MetaEvent::SetTempo,
    ///         data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
    ///     },
    ///     Message::MetaEvent {
    ///         delta_time: 0,
    ///         event: MetaEvent::EndOfTrack,
    ///         data: Vec::new(),
    ///     },
    ///     Message::TrackChange,
    ///     Message::MidiEvent {
    ///         delta_time: 0,
    ///         event: MidiEvent::NoteOn { ch: 0, note: 0x3c, velocity: 0x7f },
    ///     },
    ///     Message::MetaEvent {
    ///         delta_time: 0,
    ///         event: MetaEvent::EndOfTrack,
    ///         data: Vec::new(),
    ///     }
    /// ];
    ///
    /// let mut writer = Writer::new();
    /// for message in &messages {
    ///     writer.push(&message);
    /// }
    /// writer.write(&path);
    /// ```
    pub fn write(&self, path: &path::Path) -> Result<(), io::Error> {
        debug!("start writing at {:?}", path);
        let mut file = io::BufWriter::new(
            fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        );
        file.write(Tag::Header.binary())?;
        file.write(&[0, 0, 0, 6])?;
        file.write(&self.format.binary())?;
        file.write_u16::<BigEndian>(self.track_number())?;
        file.write_u16::<BigEndian>(self.time_base)?;
        let mut track_len_filo = self.track_len_filo();
        if self.messages.len() > 0 && *self.messages[0] != Message::TrackChange {
            file.write(&Message::TrackChange.binary())?;
            file.write_u32::<BigEndian>(track_len_filo.pop().unwrap() as u32)?;
        }
        let mut pre_status_byte: Option<u8> = None;
        for message in &self.messages {
            match **message {
                Message::TrackChange => {
                    file.write(&Message::TrackChange.binary())?;
                    file.write_u32::<BigEndian>(track_len_filo.pop().unwrap() as u32)?;
                    pre_status_byte = None;
                    debug!("wrote track change");
                }
                Message::MidiEvent {
                    delta_time,
                    ref event,
                    ..
                } => {
                    let delta_time = VLQ::new(delta_time);
                    let tmp_status_byte = (*event).status_byte();
                    match pre_status_byte {
                        Some(pre_status_byte) if pre_status_byte == tmp_status_byte => {
                            let tmp_message = message.binary();
                            file.write(&tmp_message[0..delta_time.len()])?;
                            file.write(&message.binary()[delta_time.len() + 1..])?;
                            debug!("wrote some message with running status");
                        }
                        _ => {
                            file.write(&message.binary())?;
                            debug!("wrote some message");
                            if self.running_status {
                                pre_status_byte = Some(tmp_status_byte);
                            }
                        }
                    };
                }
                _ => {
                    file.write(&message.binary())?;
                    debug!("wrote some message");
                }
            }
        }
        Ok(file.flush()?)
    }
    fn track_len_filo(&self) -> Vec<usize> {
        // First In Last Out
        let mut tracks_len: Vec<usize> = vec![0];
        let mut pre_status_byte: Option<u8> = None;
        for message in &self.messages {
            match **message {
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
        let mut number = if self.messages.len() > 0 && *self.messages[0] != Message::TrackChange {
            1
        } else {
            0
        };
        for message in &self.messages {
            number += match **message {
                Message::TrackChange => 1,
                _ => 0,
            };
        }
        number
    }
}
