use formats::*;
use std::fmt;

/// Common methods among three SMF Events.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{MessageTool, MetaEvent, MidiEvent, SysExEvent};
///
/// assert_eq!(MetaEvent::EndOfTrack.binary(), [0xff, 0x2f]);
/// assert_eq!(MidiEvent::NoteOn { ch: 0x03, note: 0x00, velocity: 0x65 }.len(), 3);
/// assert_eq!(SysExEvent::F0.status_byte(), 0xf0);
/// ```
pub trait MessageTool {
    /// Returns message's binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{MessageTool, MetaEvent, MidiEvent, SysExEvent};
    ///
    /// assert_eq!(MetaEvent::Lyric.binary(), [0xff, 0x05]);
    /// assert_eq!(
    ///     MidiEvent::NoteOff { ch: 0x04, note: 0x02, velocity: 0x00 }.binary(),
    ///     [0x84, 0x02, 0x00]
    /// );
    /// assert_eq!(SysExEvent::F7.binary(), [0xf7]);
    /// ```
    fn binary(&self) -> Vec<u8>;
    /// Returns length of message's binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{MessageTool, MetaEvent, MidiEvent, SysExEvent};
    ///
    /// assert_eq!(MetaEvent::InstrumentName.len(), 2);
    /// assert_eq!(MidiEvent::ChannelPressure { ch: 0x05, pressure: 0x45 }.len(), 2);
    /// assert_eq!(SysExEvent::F0.len(), 1);
    /// ```
    fn len(&self) -> usize;
    /// Returns message's status byte for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{MessageTool, MetaEvent, MidiEvent, SysExEvent};
    ///
    /// assert_eq!(MetaEvent::Lyric.status_byte(), 0xff);
    /// assert_eq!(MidiEvent::ProgramChange { ch: 0x01, program: 0x03 }.status_byte(), 0xc1);
    /// assert_eq!(SysExEvent::F7.status_byte(), 0xf7);
    /// ```
    fn status_byte(&self) -> u8;
}

/// An enum representing three SMF events and track change event.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{Message, MetaEvent};
/// use ghakuf::formats::VLQ;
///
/// let mut messages: Vec<Message> = Vec::new();
/// messages.push(Message::MetaEvent {
///     delta_time: VLQ::new(0),
///     event: MetaEvent::Lyric,
///     data: b"aitakute_aitakute_furufuru".to_vec(),
/// });
/// messages.push(Message::TrackChange);
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    MetaEvent {
        delta_time: VLQ,
        event: MetaEvent,
        data: Vec<u8>,
    },
    MidiEvent { delta_time: VLQ, event: MidiEvent },
    SysExEvent {
        delta_time: VLQ,
        event: SysExEvent,
        data: Vec<u8>,
    },
    TrackChange,
}
impl Message {
    /// Returns binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{Message, MidiEvent};
    /// use ghakuf::formats::VLQ;
    ///
    /// assert_eq!(
    ///     Message::MidiEvent {
    ///         delta_time: VLQ::new(0),
    ///         event: MidiEvent::NoteOn { ch: 0x01, note: 0x3c, velocity: 0x7f }
    ///     }.binary(),
    ///    vec![0x00, 0x91, 0x3c, 0x7f]
    /// );
    /// ```
    pub fn binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        use messages::Message::*;
        match *self {
            MetaEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
                binary.extend_from_slice(&VLQ::new(data.len() as u32).binary());
                binary.extend_from_slice(&data);
            }
            MidiEvent {
                delta_time,
                ref event,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
            }
            SysExEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
                use messages::SysExEvent::*;
                match *event {
                    F0 => {
                        binary.append(&mut VLQ::new(data.len() as u32 - 1).binary());
                        binary.extend_from_slice(&data[1..]);
                    }
                    _ => {
                        binary.append(&mut VLQ::new(data.len() as u32).binary());
                        binary.extend_from_slice(&data);
                    }
                }
            }
            TrackChange => binary.append(&mut Tag::Track.binary().to_vec()),
        }
        binary
    }
    /// Return binary array length of message.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{Message, MetaEvent};
    /// use ghakuf::formats::VLQ;
    ///
    /// assert_eq!(
    ///     Message::MetaEvent {
    ///         delta_time: VLQ::new(0),
    ///         event: MetaEvent::Lyric,
    ///         data: b"aitanakatta_aitanakatta_no!".to_vec(),
    ///     }.len(),
    ///    31
    /// );
    /// ```
    pub fn len(&self) -> usize {
        use messages::Message::*;
        match *self {
            MetaEvent {
                delta_time,
                ref event,
                ref data,
            } => delta_time.len() + event.len() + (VLQ::new(data.len() as u32).len()) + data.len(),
            MidiEvent {
                delta_time,
                ref event,
            } => delta_time.len() + event.len(),
            SysExEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                use messages::SysExEvent::*;
                delta_time.len() +
                    VLQ::new(
                        data.len() as u32 +
                            match *event {
                                F0 => 1,
                                _ => 0,
                            },
                    ).len() + data.len()
            }
            TrackChange => Tag::Track.binary().len(),
        }
    }
}
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use messages::Message::*;
        match *self {
            MetaEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                write!(
                    f,
                    "(delta time: {:>4}, Meta Event: {}, data: {:?})",
                    delta_time,
                    event,
                    data
                )
            }
            MidiEvent {
                delta_time,
                ref event,
            } => {
                write!(
                    f,
                    "(delta time: {:>4}, MIDI Event: {})",
                    delta_time,
                    event,
                )
            }
            SysExEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                write!(
                    f,
                    "(delta time: {:>4}, System Exclusive Event: {}, data: {:?})",
                    delta_time,
                    event,
                    data
                )
            }
            TrackChange => {
                write!(
                    f,
                    "Track Change",
                )
            }
        }
    }
}

/// An enum representing Meta event of SMF.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{MessageTool, MetaEvent};
///
/// let event: MetaEvent = MetaEvent::SetTempo;
/// assert_eq!(event.binary(), [0xff, 0x51]);
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum MetaEvent {
    SequenceNumber,
    TextEvent,
    CopyrightNotice,
    SequenceOrTrackName,
    InstrumentName,
    Lyric,
    Marker,
    CuePoint,
    MIDIChannelPrefix,
    EndOfTrack,
    SetTempo,
    SMTPEOffset,
    TimeSignature,
    KeySignature,
    SequencerSpecificMetaEvent,
    Unknown { event_type: u8 },
}
impl MetaEvent {
    /// Builds MetaEvent from status value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::MetaEvent;
    ///
    /// let event: MetaEvent = MetaEvent::new(0x06);
    /// assert_eq!(event, MetaEvent::Marker);
    /// ```
    pub fn new(event_type: u8) -> MetaEvent {
        match event_type {
            0x00 => MetaEvent::SequenceNumber,
            0x01 => MetaEvent::TextEvent,
            0x02 => MetaEvent::CopyrightNotice,
            0x03 => MetaEvent::SequenceOrTrackName,
            0x04 => MetaEvent::InstrumentName,
            0x05 => MetaEvent::Lyric,
            0x06 => MetaEvent::Marker,
            0x07 => MetaEvent::CuePoint,
            0x20 => MetaEvent::MIDIChannelPrefix,
            0x2F => MetaEvent::EndOfTrack,
            0x51 => MetaEvent::SetTempo,
            0x54 => MetaEvent::SMTPEOffset,
            0x58 => MetaEvent::TimeSignature,
            0x59 => MetaEvent::KeySignature,
            0x7F => MetaEvent::SequencerSpecificMetaEvent,
            _ => MetaEvent::Unknown { event_type },
        }
    }
}
impl MessageTool for MetaEvent {
    fn binary(&self) -> Vec<u8> {
        use messages::MetaEvent::*;
        vec![
            self.status_byte(),
            match *self {
                SequenceNumber => 0x00,
                TextEvent => 0x01,
                CopyrightNotice => 0x02,
                SequenceOrTrackName => 0x03,
                InstrumentName => 0x04,
                Lyric => 0x05,
                Marker => 0x06,
                CuePoint => 0x07,
                MIDIChannelPrefix => 0x20,
                EndOfTrack => 0x2F,
                SetTempo => 0x51,
                SMTPEOffset => 0x54,
                TimeSignature => 0x58,
                KeySignature => 0x59,
                SequencerSpecificMetaEvent => 0x7F,
                Unknown { event_type } => event_type,
            },
        ]
    }
    fn len(&self) -> usize {
        2
    }
    fn status_byte(&self) -> u8 {
        0xff
    }
}
impl fmt::Display for MetaEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use messages::MetaEvent::*;
        match *self {
            SequenceNumber => write!(f, "SequenceNumber"),
            TextEvent => write!(f, "TextEvent"),
            CopyrightNotice => write!(f, "CopyrightNotice"),
            SequenceOrTrackName => write!(f, "SequenceOrTrackName"),
            InstrumentName => write!(f, "InstrumentName"),
            Lyric => write!(f, "Lyric"),
            Marker => write!(f, "Marker"),
            CuePoint => write!(f, "CuePoint"),
            MIDIChannelPrefix => write!(f, "MIDIChannelPrefix"),
            EndOfTrack => write!(f, "EndOfTrack"),
            SetTempo => write!(f, "SetTempo"),
            SMTPEOffset => write!(f, "SMTPEOffset"),
            TimeSignature => write!(f, "TimeSignature"),
            KeySignature => write!(f, "KeySignature"),
            SequencerSpecificMetaEvent => write!(f, "SequencerSpecificMetaEvent"),
            Unknown { event_type } => write!(f, "(Unknown, event type: {}", event_type),
        }
    }
}

/// An enum representing Midi event of SMF.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{MessageTool, MidiEvent};
///
/// let event: MidiEvent = MidiEvent::NoteOff { ch: 0x04, note: 0x02, velocity: 0x00 };
/// assert_eq!(event.binary(), [0x84, 0x02, 0x00]);
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum MidiEvent {
    NoteOff { ch: u8, note: u8, velocity: u8 },
    NoteOn { ch: u8, note: u8, velocity: u8 },
    PolyphonicKeyPressure { ch: u8, note: u8, velocity: u8 },
    ControlChange { ch: u8, control: u8, data: u8 },
    ProgramChange { ch: u8, program: u8 },
    ChannelPressure { ch: u8, pressure: u8 },
    PitchBendChange { ch: u8, data: i16 },
    Unknown { ch: u8 },
}
/// A struct for building Midi event.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{MidiEvent, MidiEventBuilder};
///
/// let mut builder = MidiEventBuilder::new(0x91);
/// builder.push(0x9c);
/// builder.push(0x13);
/// let event: MidiEvent = builder.build();
///
/// assert_eq!(
///     MidiEvent::NoteOn { ch: 0x01, note: 0x9c, velocity: 0x13 },
///     event
/// )
/// ```
pub struct MidiEventBuilder {
    status: u8,
    shortage: u8,
    data: Vec<u8>,
}
impl MidiEventBuilder {
    /// Builds MidiEventBuilder from Midi status.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{MidiEvent, MidiEventBuilder};
    ///
    /// let mut builder = MidiEventBuilder::new(0x91);
    /// ```
    pub fn new(status: u8) -> MidiEventBuilder {
        MidiEventBuilder {
            status: status,
            data: Vec::new(),
            shortage: match status & 0xf0 {
                0x80...0xb0 | 0xe0 => 2,
                0xc0 | 0xd0 => 1,
                _ => 0,
            },
        }
    }
    /// Pushed u8 value to MidiEventBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::MidiEventBuilder;
    ///
    /// let mut builder = MidiEventBuilder::new(0x91);
    /// builder.push(0x00);
    /// builder.push(0x01);
    /// ```
    ///
    /// *Note*: MidiEventBuilder can accept only 2 or 3 u8 value due to SMF restriction.
    pub fn push(&mut self, data: u8) {
        if self.shortage > 0 {
            self.data.push(data);
            self.shortage -= 1;
        } else {
            warn!("Your data was ignored. MidiEventBuilder can accept only 2 or 3 u8 value due to SMF restriction.");
        }
    }
    /// Returns num till MidiEventBuilder saturated.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::MidiEventBuilder;
    ///
    /// let mut builder = MidiEventBuilder::new(0x91);
    /// assert_eq!(builder.shortage(), 2);
    /// builder.push(0x00);
    /// assert_eq!(builder.shortage(), 1);
    /// builder.push(0x02);
    /// assert_eq!(builder.shortage(), 0);
    ///
    /// let mut builder = MidiEventBuilder::new(0xc0);
    /// assert_eq!(builder.shortage(), 1);
    /// builder.push(0x00);
    /// assert_eq!(builder.shortage(), 0);
    /// ```
    pub fn shortage(&self) -> u8 {
        self.shortage
    }
    /// Builds MidiEvnet from MidiEventBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::{MidiEvent, MidiEventBuilder};
    ///
    /// let mut builder = MidiEventBuilder::new(0x91);
    /// builder.push(0x00);
    /// builder.push(0x01);
    /// builder.push(0x02);
    /// let event: MidiEvent = builder.build();
    /// ```
    pub fn build(&self) -> MidiEvent {
        match self.status & 0xf0 {
            0x80 => {
                MidiEvent::NoteOff {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0x90 => {
                MidiEvent::NoteOn {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0xa0 => {
                MidiEvent::PolyphonicKeyPressure {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0xb0 => {
                MidiEvent::ControlChange {
                    ch: self.status & 0x0f,
                    control: self.data[0],
                    data: self.data[1],
                }
            }
            0xc0 => {
                MidiEvent::ProgramChange {
                    ch: self.status & 0x0f,
                    program: self.data[0],
                }
            }
            0xd0 => {
                MidiEvent::ChannelPressure {
                    ch: self.status & 0x0f,
                    pressure: self.data[0],
                }
            }
            0xe0 => {
                let lsb = self.data[0] as u16;
                let msb = (self.data[1] as u16) << 8;
                MidiEvent::PitchBendChange {
                    ch: self.status & 0x0f,
                    data: (msb & lsb) as i16 - 8192,
                }
            }
            _ => MidiEvent::Unknown { ch: self.status & 0x0f },
        }
    }
}
impl MessageTool for MidiEvent {
    fn binary(&self) -> Vec<u8> {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { note, velocity, .. } => vec![self.status_byte(), note, velocity],
            NoteOn { note, velocity, .. } => vec![self.status_byte(), note, velocity],
            PolyphonicKeyPressure { note, velocity, .. } => vec![self.status_byte(), note, velocity],
            ControlChange { control, data, .. } => vec![self.status_byte(), control, data],
            ProgramChange { program, .. } => vec![self.status_byte(), program],
            ChannelPressure { pressure, .. } => vec![self.status_byte(), pressure],
            MidiEvent::PitchBendChange { data, .. } => {
                let pitch_bend: u16 = (data + 8192) as u16;
                vec![
                    self.status_byte(),
                    (pitch_bend >> 7) as u8,
                    (pitch_bend & 0b1111111) as u8,
                ]
            }
            MidiEvent::Unknown { .. } => vec![self.status_byte()],
        }
    }
    fn len(&self) -> usize {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { .. } |
            NoteOn { .. } |
            PolyphonicKeyPressure { .. } |
            ControlChange { .. } |
            PitchBendChange { .. } => 3,
            ProgramChange { .. } |
            ChannelPressure { .. } => 2,
            Unknown { .. } => 1,
        }
    }
    fn status_byte(&self) -> u8 {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { ch, .. } => 0x80 | (ch & 0x0f),
            NoteOn { ch, .. } => 0x90 | (ch & 0x0f),
            PolyphonicKeyPressure { ch, .. } => 0xa0 | (ch & 0x0f),
            ControlChange { ch, .. } => 0xb0 | (ch & 0x0f),
            ProgramChange { ch, .. } => 0xc0 | (ch & 0x0f),
            ChannelPressure { ch, .. } => 0xd0 | (ch & 0x0f),
            PitchBendChange { ch, .. } => 0xe0 | (ch & 0x0f),
            Unknown { ch } => 0x80 | (ch & 0x0f),
        }
    }
}
impl fmt::Display for MidiEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { ch, note, velocity } => {
                write!(
                    f,
                    "(NoteOff{{ch: {}, note: {}, velocity: {}}})",
                    ch,
                    note,
                    velocity
                )
            }
            NoteOn { ch, note, velocity } => {
                write!(
                    f,
                    "(NoteOn{{ch: {}, note: {}, velocity: {}}})",
                    ch,
                    note,
                    velocity
                )
            }
            PolyphonicKeyPressure { ch, note, velocity } => {
                write!(
                    f,
                    "(PolyphonicKeyPressure{{ch: {}, note: {}, velocity: {}}})",
                    ch,
                    note,
                    velocity
                )
            }
            ControlChange { ch, control, data } => {
                write!(
                    f,
                    "(ControlChange{{ch: {}, control: {}, data: {}}})",
                    ch,
                    control,
                    data
                )
            }
            ProgramChange { ch, program } => write!(f, "(ProgramChange{{ch: {}, program: {}}})", ch, program),
            ChannelPressure { ch, pressure } => write!(f, "(ChannelPressure{{ch: {}, pressure: {}}})", ch, pressure),
            PitchBendChange { ch, data } => write!(f, "(PitchBendChange{{ch: {}, data: {}}})", ch, data),
            Unknown { ch } => write!(f, "(Unknown{{ch: {}}})", ch),
        }
    }
}

/// An enum representing System Exclusive event of SMF.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::{MessageTool, SysExEvent};
///
/// let event: SysExEvent = SysExEvent::F0;
/// assert_eq!(event.status_byte(), 0xf0);
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum SysExEvent {
    F0,
    F7,
    Unknown { status: u8 },
}
impl SysExEvent {
    /// Builds SysExEvent from status value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::SysExEvent;
    ///
    /// let event: SysExEvent = SysExEvent::new(0xf0);
    /// assert_eq!(event, SysExEvent::F0);
    /// ```
    pub fn new(status: u8) -> SysExEvent {
        use messages::SysExEvent::*;
        match status {
            0xF0 => F0,
            0xF7 => F7,
            _ => Unknown { status: status },
        }
    }
}
impl MessageTool for SysExEvent {
    fn binary(&self) -> Vec<u8> {
        vec![self.status_byte()]
    }
    fn len(&self) -> usize {
        1
    }
    fn status_byte(&self) -> u8 {
        match *self {
            SysExEvent::F0 { .. } => 0xf0,
            SysExEvent::F7 { .. } => 0xf7,
            SysExEvent::Unknown { status, .. } => status,
        }
    }
}
impl fmt::Display for SysExEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use messages::SysExEvent::*;
        match *self {
            F0 => write!(f, "F0"),
            F7 => write!(f, "F7"),
            Unknown { status } => write!(f, "(Unknown, status: {}", status),
        }
    }
}
