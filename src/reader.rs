use byteorder::{BigEndian, ReadBytesExt};
use formats::*;
use messages::*;
use std::error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::fmt;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::mem::size_of;
use std::path;
use std::path::PathBuf;

/// `ghakuf`'s SMF parser.
///
/// # Examples
///
/// ```
/// use ghakuf::formats::*;
/// use ghakuf::messages::*;
/// use ghakuf::reader::handler::*;
/// use ghakuf::reader::reader::*;
/// use std::path::PathBuf;
///
/// let mut reader = Reader::new(
///     Box::new(HogeHandler {}),
///     PathBuf::from("tests/test.mid"),
/// ).unwrap();
/// reader.read();
///
/// struct HogeHandler {}
/// impl Handler for HogeHandler {
///     fn header(&mut self, format: Format, track: u16, time_base: u16) {}
///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {}
///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {}
///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {}
///     fn track_change(&mut self) {}
/// }
/// ```
pub struct Reader {
    file: BufReader<File>,
    handlers: Vec<Box<Handler>>,
    path: PathBuf,
}
impl Reader {
    /// Builds Reader with handler(observer) and SMF file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::*;
    /// use ghakuf::messages::*;
    /// use ghakuf::reader::handler::*;
    /// use ghakuf::reader::reader::*;
    /// use std::path::PathBuf;
    ///
    /// let mut reader = Reader::new(
    ///     Box::new(FugaHandler {}),
    ///     PathBuf::from("tests/test.mid"),
    /// );
    ///
    /// struct FugaHandler {}
    /// impl Handler for FugaHandler {
    ///     fn header(&mut self, format: Format, track: u16, time_base: u16) {}
    ///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {}
    ///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {}
    ///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {}
    ///     fn track_change(&mut self) {}
    /// }
    /// ```
    pub fn new(handler: Box<Handler>, path: PathBuf) -> Result<Reader, ReadError> {
        let mut handlers = Vec::new();
        handlers.push(handler);
        Ok(Reader {
            file: BufReader::new(OpenOptions::new().read(true).open(&path)?),
            path: path,
            handlers: handlers,
        })
    }
    /// Pushes Handler to Reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::*;
    /// use ghakuf::messages::*;
    /// use ghakuf::reader::handler::*;
    /// use ghakuf::reader::reader::*;
    /// use std::path::PathBuf;
    ///
    /// let mut reader = Reader::new(
    ///     Box::new(FugaHandler {}),
    ///     PathBuf::from("tests/test.mid"),
    /// ).unwrap();
    /// reader.push_hanlder(Box::new(NyanHandler {}));
    ///
    /// struct FugaHandler {}
    /// impl Handler for FugaHandler {
    ///     fn header(&mut self, format: Format, track: u16, time_base: u16) {}
    ///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {}
    ///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {}
    ///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {}
    ///     fn track_change(&mut self) {}
    /// }
    ///
    /// struct NyanHandler {}
    /// impl Handler for NyanHandler {
    ///     fn header(&mut self, format: Format, track: u16, time_base: u16) {}
    ///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {}
    ///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {}
    ///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {}
    ///     fn track_change(&mut self) {}
    /// }
    /// ```
    pub fn push_hanlder(&mut self, handler: Box<Handler>) {
        self.handlers.push(handler);
    }
    /// Parses SMF messages and fires(broadcasts) handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::*;
    /// use ghakuf::messages::*;
    /// use ghakuf::reader::handler::*;
    /// use ghakuf::reader::reader::*;
    /// use std::path::PathBuf;
    ///
    /// let mut reader = Reader::new(
    ///     Box::new(HogeHandler {}),
    ///     PathBuf::from("tests/test.mid"),
    /// ).unwrap();
    /// reader.read();
    ///
    /// struct HogeHandler {}
    /// impl Handler for HogeHandler {
    ///     fn header(&mut self, format: Format, track: u16, time_base: u16) {}
    ///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {}
    ///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {}
    ///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {}
    ///     fn track_change(&mut self) {}
    /// }
    /// ```
    pub fn read(&mut self) -> Result<(), ReadError> {
        self.file.seek(SeekFrom::Start(0))?;
        self.check_tag(Tag::Header)?;
        self.read_header_block()?;
        while self.check_tag(Tag::Track)? {
            self.read_track_block()?;
        }
        Ok(())
    }
    fn check_tag(&mut self, tag_type: Tag) -> Result<bool, ReadError> {
        let mut tag = [0u8; 4];
        let buf_size = self.file.read(&mut tag)?;
        if tag_type.binary() == &tag || (buf_size == 0 && tag_type == Tag::Track) {
            if tag_type == Tag::Track {
                for handler in &mut self.handlers {
                    handler.track_change();
                }
            }
            Ok(buf_size > 0)
        } else {
            error!("invalid tag has found: {:?}", &tag);
            match tag_type {
                Tag::Header => {
                    return Err(ReadError::InvalidHeaderTag {
                        tag: tag,
                        path: self.path.clone(),
                    })
                }
                Tag::Track => {
                    return Err(ReadError::InvalidTrackTag {
                        tag: tag,
                        path: self.path.clone(),
                    })
                }
            }

        }
    }
    fn read_header_block(&mut self) -> Result<&mut Reader, ReadError> {
        let file_code = self.file.read_u32::<BigEndian>()?;
        if file_code == 6u32 {
            let format = Format::new(self.file.read_u16::<BigEndian>()?);
            let track = self.file.read_u16::<BigEndian>()?;
            let timebase = self.file.read_u16::<BigEndian>()?;
            for handler in &mut self.handlers {
                handler.header(format, track, timebase);
            }
            Ok(self)
        } else {
            error!("invalid smf identify code has found at header");
            Err(ReadError::InvalidIdentifyCode {
                code: file_code,
                path: self.path.clone(),
            })
        }
    }
    fn read_track_block(&mut self) -> Result<&mut Reader, ReadError> {
        let mut data_size = self.file.read_u32::<BigEndian>()?;
        let mut pre_status: u8 = 0;
        while data_size > 0 {
            let delta_time = self.read_vlq()?;
            data_size -= delta_time.len() as u32;
            let mut status = self.file.read_u8()?;
            if status < 0b10000000 {
                info!(
                    "running status has found! recorded data: {}, corrected data: {}",
                    status,
                    pre_status
                );
                status = pre_status;
                self.file.seek(SeekFrom::Current(-1))?;
            } else {
                data_size -= size_of::<u8>() as u32;
            }
            match status {
                0xff => {
                    // meta event
                    info!("meta event status has found!");
                    let meta_event = MetaEvent::new(self.file.read_u8()?);
                    data_size -= size_of::<u8>() as u32;
                    let len = self.read_vlq()?;
                    let data = self.read_data(&len)?;
                    data_size -= len.len() as u32 + len.val();
                    for handler in &mut self.handlers {
                        handler.meta_event(delta_time.val(), &meta_event, &data);
                    }
                }
                0x80...0xef => {
                    // midi event
                    info!("midi event status has found!");
                    let mut builder = MidiEventBuilder::new(status);
                    while builder.shortage() > 0 {
                        builder.push(self.file.read_u8()?);
                        data_size -= size_of::<u8>() as u32;
                    }
                    let midi_event = builder.build();
                    for handler in &mut self.handlers {
                        handler.midi_event(delta_time.val(), &midi_event);
                    }
                    pre_status = status;
                }
                0xf0 | 0xf7 => {
                    // system exclusive event
                    info!("system exclusice event status has found!");
                    if status == 0xf7 && pre_status == 0xf0 {
                        pre_status = 0;
                        data_size -= size_of::<u8>() as u32;
                        continue;
                    }
                    let sys_ex_event = SysExEvent::new(status);
                    let len = self.read_vlq()?;
                    let data = self.read_data(&len)?;
                    data_size -= len.len() as u32 + len.val();
                    for handler in &mut self.handlers {
                        handler.sys_ex_event(delta_time.val(), &sys_ex_event, &data);
                    }
                    if status == 0xf0 {
                        pre_status = 0xf0;
                    }
                }
                _ => {
                    error!("unknown status has found: {}", status);
                    return Err(ReadError::UnknownMessageStatus {
                        status: status,
                        path: self.path.clone(),
                    });
                }
            };
        }
        Ok(self)
    }
    fn read_vlq(&mut self) -> Result<VLQ, ReadError> {
        let mut vlq_builder = VLQBuilder::new();
        while !vlq_builder.closed() {
            vlq_builder.push(self.file.read_u8()?);
        }
        Ok(vlq_builder.build())
    }
    fn read_data(&mut self, vlq: &VLQ) -> Result<Vec<u8>, ReadError> {
        let len = vlq.val();
        let mut data: Vec<u8> = Vec::new();
        data.reserve(len as usize);
        for _ in 0..len {
            data.push(self.file.read_u8()?);
        }
        Ok(data)
    }
}

///  Handler(Observer) of reader::reader::Reader.
///
///  # Examples
///
///  ```
///  use ghakuf::formats::*;
///  use ghakuf::messages::*;
///  use ghakuf::reader::handler::*;
///
///  struct HogeHandler {}
///  impl Handler for HogeHandler {
///      fn header(&mut self, format: Format, track: u16, time_base: u16) {
///        // Something
///      }
///      fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
///        // you
///      }
///      fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
///        // want
///      }
///      fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
///        // to
///      }
///      fn track_change(&mut self) {
///        // do
///      }
///  }
///  ```
pub trait Handler {
    /// Fired when SMF header track has found.
    fn header(&mut self, format: Format, track: u16, time_base: u16);
    /// Fired when meta event has found.
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>);
    /// Fired when MIDI event has found.
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent);
    /// Fired when system evclusive event has found.
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>);
    /// Fired when track has changed.
    fn track_change(&mut self);
}

/// An enum represents errors of SMF parser.
#[derive(Debug)]
pub enum ReadError {
    /// Reads tag error with invalid tag and file path at header.
    InvalidHeaderTag { tag: [u8; 4], path: PathBuf },
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    InvalidIdentifyCode { code: u32, path: PathBuf },
    /// Reads tag error with invalid tag and file path at track.
    InvalidTrackTag { tag: [u8; 4], path: PathBuf },
    /// Standard file IO error (std::io::Error)
    Io(io::Error),
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    UnknownMessageStatus { status: u8, path: path::PathBuf },
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use reader::ReadError::*;
        match *self {
            InvalidHeaderTag { tag, ref path } => {
                write!(
                    f,
                    "Invalid header tag '{:?}' has found: {}",
                    tag,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            InvalidIdentifyCode { code, ref path } => {
                write!(
                    f,
                    "Invalid identify code '{}' has found at header: {}",
                    code,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            InvalidTrackTag { tag, ref path } => {
                write!(
                    f,
                    "Invalid track tag '{:?}' has found: {}",
                    tag,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            Io(ref err) => err.fmt(f),
            UnknownMessageStatus { status, ref path } => {
                write!(
                    f,
                    "Unknown message status '{:x}' has found: {}",
                    status,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
        }
    }
}
impl error::Error for ReadError {
    fn description(&self) -> &str {
        use reader::ReadError::*;
        match *self {
            InvalidHeaderTag { .. } => "Invalid header tag has found. This file dosen't follow SMF format.",
            InvalidIdentifyCode { .. } => {
                concat!(
                    "Invalid SMF identify code has found at header.",
                    "This file dosen't follow SMF format."
                )
            }
            InvalidTrackTag { .. } => "Invalid track tag has found. This file dosen't follow SMF format.",
            ReadError::Io(ref err) => err.description(),
            UnknownMessageStatus { .. } => "Unknown message status has found. This file dosen't follow SMF format.",
        }
    }
}
impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}
