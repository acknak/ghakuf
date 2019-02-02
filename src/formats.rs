use std::fmt;

/// An enum representing SMF format(0-2).
///
/// # Examples
///
/// ```
/// use ghakuf::formats::Format;
///
/// let format = Format::new(0x0001);
/// assert_eq!(format.binary(), [0x00, 0x01]);
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Format {
    /// Format 0 (binary literal: [0x00, 0x00])
    F0,
    /// Format 1 (binary literal: [0x00, 0x01])
    F1,
    /// Format 2 (binary literal: [0x00, 0x02])
    F2,
    /// Unknown format (at binary literal, [0xff, 0xff])
    Unknown,
}
impl Format {
    /// Builds Format from u16 value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::Format;
    ///
    /// let format = Format::new(0x0001);
    /// assert_eq!(format, Format::F1);
    /// ```
    pub fn new(format: u16) -> Format {
        use formats::Format::*;
        match format {
            0 => F0,
            1 => F1,
            2 => F2,
            _ => Unknown,
        }
    }
    /// Makes binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::Format;
    ///
    /// assert_eq!(Format::F2.binary(), [0x00, 0x02]);
    /// ```
    pub fn binary(&self) -> [u8; 2] {
        match *self {
            Format::F0 => [0, 0],
            Format::F1 => [0, 1],
            Format::F2 => [0, 2],
            Format::Unknown => [0xff, 0xff],
        }
    }
}
impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use formats::Format::*;
        write!(
            f,
            "{}",
            match *self {
                F0 => "Format 0",
                F1 => "Format 1",
                F2 => "Format 2",
                Unknown => "Unknown Format",
            }
        )
    }
}

/// An enum representing SMF Tag "MThd" and "MTrk".
///
/// # Examples
///
/// ```
/// use ghakuf::formats::Tag;
///
/// let tag = Tag::Header;
/// assert_eq!(tag.binary(), b"MThd");
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tag {
    /// Header tag (binary literal: b"MThd")
    Header,
    /// Format 2 (binary literal: b"MTrk")
    Track,
}
impl Tag {
    /// Makes binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::Tag;
    ///
    /// assert_eq!(Tag::Track.binary(), b"MTrk");
    /// ```
    pub fn binary(&self) -> &[u8; 4] {
        match *self {
            Tag::Header => b"MThd",
            Tag::Track => b"MTrk",
        }
    }
}

/// A struct representing SMF Variable Length Quantity.
///
/// You can make VLQ from u32 value.
///
/// # Examples
///
/// ```
/// use ghakuf::formats::VLQ;
///
/// let vlq = VLQ::new(192);
/// assert_eq!(vlq.val(), 192);
/// assert_eq!(vlq.binary(), [0x81, 0x40]);
/// assert_eq!(vlq.len(), 2);
/// ```
/// Also you can make VLQ from u8 values.
/// # Examples
/// ```
/// use ghakuf::formats::{VLQ, VLQBuilder};
///
/// let vlq = VLQBuilder::new()
///     .push(0x86)
///     .push(0xc3)
///     .push(0x17)
///     .build();
/// assert_eq!(vlq.val(), 106_903);
/// assert_eq!(vlq.binary(), [0x86, 0xc3, 0x17]);
/// assert_eq!(vlq.len(), 3);
/// ```
///
/// *Note*: Due to SMF restriction, this struct can only represent value under 2^28.
#[derive(PartialEq, Clone, Copy)]
pub struct VLQ {
    val: u32,
}
impl VLQ {
    /// Builds VLQ from u32 value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQ;
    ///
    /// let vlq: VLQ = VLQ::new(192);
    /// ```
    pub fn new(val: u32) -> VLQ {
        VLQ { val: val }
    }
    /// Makes binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQ;
    ///
    /// let vlq = VLQ::new(192);
    /// assert_eq!(vlq.binary(), [0x81, 0x40]);
    /// ```
    pub fn binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        let vlq_limit = VLQ::limit_size() - 1;
        for i in 0..VLQ::limit_size() {
            let shiftsize = 7 * (vlq_limit - i);
            let mask = 0b1111111 << shiftsize;
            if self.val & mask > 0 || binary.len() > 0 {
                let tmp = ((self.val & mask) >> shiftsize) as u8;
                binary.push(if i < vlq_limit { tmp | 0b10000000 } else { tmp });
            }
        }
        if binary.len() == 0 {
            binary.push(0);
        }
        binary
    }
    /// Returns length of binary array for SMF.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQ;
    ///
    /// let vlq = VLQ::new(192);
    /// assert_eq!(vlq.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        let mut len: usize = 1;
        let vlq_limit = VLQ::limit_size() - 1;
        for i in 0..vlq_limit {
            if self.val >> (7 * (vlq_limit - i)) > 0 {
                len = VLQ::limit_size() - i;
                break;
            }
        }
        len
    }
    /// Returns value from VLQ.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQ;
    ///
    /// let vlq = VLQ::new(192);
    /// assert_eq!(vlq.val(), 192);
    /// ```
    pub fn val(&self) -> u32 {
        self.val
    }
    fn limit_size() -> usize {
        4
    }
}
impl fmt::Debug for VLQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VLQ val: {}, binary: {:?}, len: {}}}",
            self.val,
            self.binary(),
            self.len()
        )
    }
}
impl fmt::Display for VLQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(VLQ: {})", self.val)
    }
}
#[cfg(test)]
mod vlq_tests {
    use formats::*;
    #[test]
    fn binary_0() {
        let tester = VLQBuilder::new().push(0x00).build();
        assert_eq!(tester.val(), 0);
        assert_eq!(tester.len(), 1);
        assert_eq!(tester.binary(), [0]);
    }
    #[test]
    fn binary_ff7f7f() {
        let tester = VLQBuilder::new().push(0xff).push(0x7f).push(0x7f).build();
        assert_eq!(tester.val(), 0b1111111_1111111);
        assert_eq!(tester.len(), 2);
        assert_eq!(tester.binary(), [0xff, 0x7f]);
    }
    #[test]
    fn binary_98327() {
        let tester = VLQBuilder::new()
            .push(134)
            .push(0b10000000)
            .push(23)
            .build();
        assert_eq!(tester.val(), 98327);
        assert_eq!(tester.len(), 3);
        assert_eq!(tester.binary(), [134, 0b10000000, 23]);
    }
    #[test]
    fn binary_ffffffff7f() {
        let tester = VLQBuilder::new()
            .push(0xff)
            .push(0xff)
            .push(0xff)
            .push(0xff)
            .push(0xff)
            .build();
        assert!(tester.val() < 0b10000000_0000000_0000000_0000000);
        assert_eq!(tester.len(), 4);
        assert_eq!(tester.binary(), [0xff, 0xff, 0xff, 0x7f]);
    }
}

/// VLQ Builder from u8 values.
///
/// # Examples
///
/// ```
/// use ghakuf::formats::{VLQ, VLQBuilder};
///
/// let vlq = VLQBuilder::new().push(0x86).push(0xc3).push(0x17).build();
/// assert_eq!(vlq.val(), 106_903);
/// assert_eq!(vlq.binary(), [0x86, 0xc3, 0x17]);
/// assert_eq!(vlq.len(), 3);
/// ```
pub struct VLQBuilder {
    val: u32,
    closed: bool,
}
impl VLQBuilder {
    /// Builds VLQBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQBuilder;
    ///
    /// let vlq_builder: VLQBuilder = VLQBuilder::new();
    /// ```
    pub fn new() -> VLQBuilder {
        VLQBuilder {
            val: 0,
            closed: false,
        }
    }
    /// Pushes u8 value for VLQ to VLQBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQBuilder;
    ///
    /// let mut vlq_builder = VLQBuilder::new();
    /// vlq_builder.push(0x86).push(0xc3).push(0x17);
    /// ```
    ///
    /// *Note*: VLQBuilder can accept only 4 u8 value due to SMF restriction. This method ignore after the fifth.
    pub fn push(&mut self, data: u8) -> &mut VLQBuilder {
        if self.closed {
            warn!("Your data was ignored. VLQBuilder can accept only 4 u8 value due to SMF restriction.");
        } else {
            self.val = self.val << 7 | ((data & 0b0111_1111) as u32);
        }
        if data & 0b00000000_00000000_00000000_10000000 == 0 || self.val > 0b1111111_1111111_1111111
        {
            self.closed = true;
            info!("VLQBuilder closed.")
        }
        self
    }
    /// Checks whether VLQBuilder is saturated or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::VLQBuilder;
    ///
    /// let mut vlq_builder = VLQBuilder::new();
    ///
    /// vlq_builder.push(0x86).push(0xc3);
    /// assert!(!vlq_builder.closed());
    ///
    /// vlq_builder.push(0x17).push(0xa2);
    /// assert!(vlq_builder.closed());
    /// ```
    pub fn closed(&self) -> bool {
        self.closed
    }
    /// Builds VLQ from VLQBuilder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghakuf::formats::{VLQ, VLQBuilder};
    ///
    /// let vlq: VLQ = VLQBuilder::new().push(0x12).build();
    /// ```
    pub fn build(&self) -> VLQ {
        VLQ { val: self.val }
    }
}
