use messages::*;
use formats::*;

pub trait Handler {
    fn header(&mut self, format: Format, track: u16, time_base: u16);
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>);
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent);
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>);
    fn track_change(&mut self);
}
