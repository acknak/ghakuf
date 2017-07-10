use messages::*;
use formats::*;

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
