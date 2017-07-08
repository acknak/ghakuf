use byteorder::{BigEndian, ReadBytesExt};
use formats::*;
use messages::*;
use reader::read_error::ReadError;
use reader::handler::Handler;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::mem::size_of;
use std::path::PathBuf;

pub struct Reader {
    file: BufReader<File>,
    handlers: Vec<Box<Handler>>,
    path: PathBuf,
}
impl Reader {
    pub fn new(handler: Box<Handler>, path: PathBuf) -> Result<Reader, ReadError> {
        let mut handlers = Vec::new();
        handlers.push(handler);
        Ok(Reader {
            file: BufReader::new(OpenOptions::new().read(true).open(&path)?),
            path: path,
            handlers: handlers,
        })
    }
    pub fn add_hanlder(&mut self, handler: Box<Handler>) {
        self.handlers.push(handler);
    }
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
            error!("invalid tag found: {:?}", &tag);
            Err(ReadError::SmfFormat(self.path.clone()))
        }
    }
    fn read_header_block(&mut self) -> Result<&mut Reader, ReadError> {
        if self.file.read_u32::<BigEndian>()? == 6u32 {
            let format = Format::new(self.file.read_u16::<BigEndian>()?);
            let track = self.file.read_u16::<BigEndian>()?;
            let timebase = self.file.read_u16::<BigEndian>()?;
            for handler in &mut self.handlers {
                handler.header(format, track, timebase);
            }
            Ok(self)
        } else {
            error!("invalid smf identify code found at header");
            Err(ReadError::SmfFormat(self.path.clone()))
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
                    "running status found! found data: {}, correct data: {}",
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
                    info!("meta event status found!");
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
                    info!("midi event status found!");
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
                    info!("system exclusice event status found!");
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
                    error!("unknown status found: {}", status);
                    return Err(ReadError::SmfFormat(self.path.clone()));
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
