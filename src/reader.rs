use byteorder::{BigEndian, ReadBytesExt};
use formats::*;
use messages::*;
use std::io::{Read, Seek, SeekFrom};
use std::{error, fmt, fs, io, mem, path};

/// `ghakuf`'s SMF parser.
///
/// # Examples
///
/// ```
/// use ghakuf::messages::*;
/// use ghakuf::reader::*;
/// use std::path;
///
/// let path = path::Path::new("tests/test.mid");
/// let mut handler = HogeHandler {};
/// let mut reader = Reader::new(&mut handler, &path).unwrap();
/// let _ = reader.read();
///
/// struct HogeHandler {}
/// impl Handler for HogeHandler {
///     fn header(&mut self, format: u16, track: u16, time_base: u16) {
///         let _ = (format, track, time_base);
///     }
///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
///         let _ = (delta_time, event, data);
///     }
///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
///         let _ = (delta_time, event);
///     }
///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
///         let _ = (delta_time, event, data);
///     }
///     fn track_change(&mut self) {}
/// }
/// ```
pub struct Reader<'a> {
    file: io::BufReader<fs::File>,
    handlers: Vec<&'a mut dyn Handler>,
    path: &'a path::Path,
}
impl<'a> Reader<'a> {
    /// Builds Reader with handler(observer) and SMF file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::reader::*;
    /// use std::path;
    ///
    /// let path = path::Path::new("tests/test.mid");
    /// let mut handler = FugaHandler {};
    /// let mut reader = Reader::new(&mut handler, &path);
    ///
    /// struct FugaHandler {}
    /// impl Handler for FugaHandler {}
    /// ```
    pub fn new(
        handler: &'a mut dyn Handler,
        path: &'a path::Path,
    ) -> Result<Reader<'a>, ReadError<'a>> {
        let mut handlers: Vec<&'a mut dyn Handler> = Vec::new();
        handlers.push(handler);
        Ok(Reader {
            file: io::BufReader::new(fs::OpenOptions::new().read(true).open(path)?),
            path: path,
            handlers: handlers,
        })
    }
    /// Pushes Handler to Reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::reader::*;
    /// use std::path;
    ///
    /// let path = path::Path::new("tests/test.mid");
    /// let mut fuga_handler = FugaHandler {};
    /// let mut nyan_handler = NyanHandler {};
    /// let mut reader = Reader::new(&mut fuga_handler, &path).unwrap();
    /// reader.push_handler(&mut nyan_handler);
    ///
    /// struct FugaHandler {}
    /// impl Handler for FugaHandler {}
    ///
    /// struct NyanHandler {}
    /// impl Handler for NyanHandler {}
    /// ```
    pub fn push_handler(&mut self, handler: &'a mut dyn Handler) {
        self.handlers.push(handler);
    }
    /// Parses SMF messages and fires(broadcasts) handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::messages::*;
    /// use ghakuf::reader::*;
    /// use std::path;
    ///
    /// let path = path::Path::new("tests/test.mid");
    /// let mut handler = HogeHandler{};
    /// let mut reader = Reader::new(&mut handler, &path).unwrap();
    /// let _ = reader.read();
    ///
    /// struct HogeHandler {}
    /// impl Handler for HogeHandler {
    ///     fn header(&mut self, format: u16, track: u16, time_base: u16) {
    ///         let _ = (format, track, time_base);
    ///     }
    ///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
    ///         let _ = (delta_time, event, data);
    ///     }
    ///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
    ///         let _ = (delta_time, event);
    ///     }
    ///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
    ///         let _ = (delta_time, event, data);
    ///     }
    ///     fn track_change(&mut self) {}
    /// }
    /// ```
    pub fn read(&mut self) -> Result<(), ReadError> {
        let mut skip = true;
        for handler in &mut self.handlers {
            skip &= handler.status() == HandlerStatus::SkipAll;
            if skip {
                return Err(ReadError::NoValidHandler);
            }
        }
        self.file.seek(io::SeekFrom::Start(0))?;
        self.check_tag(Tag::Header)?;
        self.read_header_block()?;
        while self.check_tag(Tag::Track)? {
            skip = true;
            for handler in &mut self.handlers {
                skip &= handler.status() == HandlerStatus::SkipAll;
            }
            if skip {
                break;
            };
            self.read_track_block()?;
        }
        Ok(())
    }
    fn check_tag(&mut self, tag_type: Tag) -> Result<bool, ReadError<'a>> {
        let mut tag = [0u8; 4];
        if self.file.read(&mut tag)? < 4 {
            match tag_type {
                Tag::Header => {
                    error!("header tag hasn't found");
                    Err(ReadError::InvalidHeaderTag {
                        tag: tag,
                        path: self.path,
                    })
                }
                Tag::Track => Ok(false),
            }
        } else if tag_type.binary() == &tag {
            match tag_type {
                Tag::Header => Ok(true),
                Tag::Track => {
                    for handler in &mut self.handlers {
                        if handler.status() != HandlerStatus::SkipAll {
                            handler.track_change();
                        }
                    }
                    Ok(true)
                }
            }
        } else {
            error!("invalid tag has found: {:?}", &tag);
            match tag_type {
                Tag::Header => Err(ReadError::InvalidHeaderTag {
                    tag: tag,
                    path: self.path,
                }),
                Tag::Track => Err(ReadError::InvalidTrackTag {
                    tag: tag,
                    path: self.path,
                }),
            }
        }
    }
    fn read_header_block(&mut self) -> Result<&mut Reader<'a>, ReadError<'a>> {
        let file_code = self.file.read_u32::<BigEndian>()?;
        if file_code == 6u32 {
            let format = self.file.read_u16::<BigEndian>()?;
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
                path: self.path,
            })
        }
    }
    fn read_track_block(&mut self) -> Result<&mut Reader<'a>, ReadError<'a>> {
        let mut data_size = self.file.read_u32::<BigEndian>()?;
        let mut pre_status: u8 = 0;
        while data_size > 0 {
            let mut skip = true;
            for handler in &mut self.handlers {
                skip &= handler.status() != HandlerStatus::Continue;
            }
            if skip {
                self.file.seek(SeekFrom::Current(data_size as i64))?;
                data_size = 0;
                continue;
            }
            let delta_time = self.read_vlq()?;
            data_size -= delta_time.len() as u32;
            let mut status = self.file.read_u8()?;
            if status < 0b10000000 {
                debug!(
                    "running status has found! recorded data: {}, corrected data: {}",
                    status, pre_status
                );
                status = pre_status;
                self.file.seek(SeekFrom::Current(-1))?;
            } else {
                data_size -= mem::size_of::<u8>() as u32;
            }
            match status {
                0xff => {
                    // meta event
                    debug!("meta event status has found!");
                    let meta_event = MetaEvent::new(self.file.read_u8()?);
                    data_size -= mem::size_of::<u8>() as u32;
                    let len = self.read_vlq()?;
                    let data = self.read_data(&len)?;
                    data_size -= len.len() as u32 + len.val();
                    for handler in &mut self.handlers {
                        if handler.status() == HandlerStatus::Continue {
                            handler.meta_event(delta_time.val(), &meta_event, &data);
                        }
                    }
                }
                0x80..=0xef => {
                    // midi event
                    debug!("midi event status has found!");
                    let mut builder = MidiEventBuilder::new(status);
                    while builder.shortage() > 0 {
                        builder.push(self.file.read_u8()?);
                        data_size -= mem::size_of::<u8>() as u32;
                    }
                    let midi_event = builder.build();
                    for handler in &mut self.handlers {
                        if handler.status() == HandlerStatus::Continue {
                            handler.midi_event(delta_time.val(), &midi_event);
                        }
                    }
                    pre_status = status;
                }
                0xf0 | 0xf7 => {
                    // system exclusive event
                    debug!("system exclusice event status has found!");
                    if status == 0xf7 && pre_status == 0xf0 {
                        pre_status = 0;
                        data_size -= mem::size_of::<u8>() as u32;
                        continue;
                    }
                    let sys_ex_event = SysExEvent::new(status);
                    let len = self.read_vlq()?;
                    let data = self.read_data(&len)?;
                    data_size -= len.len() as u32 + len.val();
                    for handler in &mut self.handlers {
                        if handler.status() == HandlerStatus::Continue {
                            handler.sys_ex_event(delta_time.val(), &sys_ex_event, &data);
                        }
                    }
                    if status == 0xf0 {
                        pre_status = 0xf0;
                    }
                }
                _ => {
                    error!("unknown status has found: {}", status);
                    return Err(ReadError::UnknownMessageStatus {
                        status: status,
                        path: self.path,
                    });
                }
            };
        }
        Ok(self)
    }
    fn read_vlq(&mut self) -> Result<VLQ, ReadError<'a>> {
        let mut vlq_builder = VLQBuilder::new();
        while !vlq_builder.closed() {
            vlq_builder.push(self.file.read_u8()?);
        }
        Ok(vlq_builder.build())
    }
    fn read_data(&mut self, vlq: &VLQ) -> Result<Vec<u8>, ReadError<'a>> {
        let len = vlq.val();
        let mut data: Vec<u8> = Vec::new();
        data.reserve(len as usize);
        for _ in 0..len {
            data.push(self.file.read_u8()?);
        }
        Ok(data)
    }
}

///  Handler(Observer) of Reader.
///
///  # Examples
///
///  ```
///  use ghakuf::messages::*;
///  use ghakuf::reader::*;
///
///  struct HogeHandler {
///    status: HandlerStatus,
///  }
///  impl HogeHandler {
///     pub fn new() -> HogeHandler {
///         HogeHandler{status: HandlerStatus::Continue}
///     }
///  }
///  impl Handler for HogeHandler {
///     fn header(&mut self, format: u16, track: u16, time_base: u16) {
///         let _ = (format, track, time_base);
///     }
///     fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
///         let _ = (delta_time, event, data);
///     }
///     fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
///         let _ = (delta_time, event);
///         self.status = HandlerStatus::SkipTrack;
///     }
///     fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
///         let _ = (delta_time, event, data);
///     }
///     fn track_change(&mut self) {
///         self.status = HandlerStatus::Continue;
///     }
///     fn status(&mut self) -> HandlerStatus {
///         self.status.clone()
///     }
///  }
///  ```
pub trait Handler {
    /// Fired when SMF header track has found.
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        let _ = (format, track, time_base);
    }
    /// Fired when meta event has found.
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        let _ = (delta_time, event, data);
    }
    /// Fired when MIDI event has found.
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        let _ = (delta_time, event);
    }
    /// Fired when system exclusive event has found.
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        let _ = (delta_time, event, data);
    }
    /// Fired when track has changed.
    fn track_change(&mut self) {}
    /// Send handler status to parser.
    fn status(&mut self) -> HandlerStatus {
        HandlerStatus::Continue
    }
}

/// An enum represents handler status.
#[derive(PartialEq, Clone, Debug)]
pub enum HandlerStatus {
    /// Continues parsing
    Continue,
    /// Skips parsing track
    SkipTrack,
    /// Skips all tracks (Parser will never send Messages to this handler any more.)
    SkipAll,
}

/// An enum represents errors of SMF parser.
#[derive(Debug)]
pub enum ReadError<'a> {
    /// Reads tag error with invalid tag and file path at header.
    InvalidHeaderTag { tag: [u8; 4], path: &'a path::Path },
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    InvalidIdentifyCode { code: u32, path: &'a path::Path },
    /// Reads tag error with invalid tag and file path at track.
    InvalidTrackTag { tag: [u8; 4], path: &'a path::Path },
    /// Standard file IO error (std::io::Error)
    Io(io::Error),
    /// Parser doesn't have any valid handlers.
    NoValidHandler,
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    UnknownMessageStatus { status: u8, path: &'a path::Path },
}
impl<'a> fmt::Display for ReadError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use reader::ReadError::*;
        match *self {
            InvalidHeaderTag { tag, ref path } => write!(
                f,
                "Invalid header tag '{:?}' has found: {}",
                tag,
                fs::canonicalize(&path).unwrap().display()
            ),
            InvalidIdentifyCode { code, ref path } => write!(
                f,
                "Invalid identify code '{}' has found at header: {}",
                code,
                fs::canonicalize(&path).unwrap().display()
            ),
            InvalidTrackTag { tag, ref path } => write!(
                f,
                "Invalid track tag '{:?}' has found: {}",
                tag,
                fs::canonicalize(&path).unwrap().display()
            ),
            Io(ref err) => err.fmt(f),
            NoValidHandler => write!(f, "Parser doesn't have any valid handlers."),
            UnknownMessageStatus { status, ref path } => write!(
                f,
                "Unknown message status '{:x}' has found: {}",
                status,
                fs::canonicalize(&path).unwrap().display()
            ),
        }
    }
}
impl<'a> error::Error for ReadError<'a> {
    fn description(&self) -> &str {
        use reader::ReadError::*;
        match *self {
            InvalidHeaderTag { .. } => "Invalid header tag has found. This file dosen't follow SMF format.",
            InvalidIdentifyCode { .. } => "Invalid SMF identify code has found at header. This file dosen't follow SMF format.",
            InvalidTrackTag { .. } => "Invalid track tag has found. This file dosen't follow SMF format.",
            ReadError::Io(ref err) => err.description(),
            NoValidHandler => "Parser doesn't have any valid handlers. Regist vailid handler.",
            UnknownMessageStatus { .. } => "Unknown message status has found. This file dosen't follow SMF format.",
        }
    }
}
impl<'a> From<io::Error> for ReadError<'a> {
    fn from(err: io::Error) -> ReadError<'a> {
        ReadError::Io(err)
    }
}
