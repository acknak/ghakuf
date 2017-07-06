#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Format {
    F0,
    F1,
    F2,
    Unknown,
}
impl Format {
    pub fn new(format: u16) -> Format {
        use formats::Format::*;
        match format {
            0 => F0,
            1 => F1,
            2 => F2,
            _ => Unknown,
        }
    }
    pub fn binary(&self) -> [u8; 2] {
        match *self {
            Format::F0 => [0, 0],
            Format::F1 => [0, 1],
            Format::F2 => [0, 2],
            Format::Unknown => [0xff, 0xff],
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tag {
    Header,
    Track,
}
impl Tag {
    pub fn binary(&self) -> &[u8; 4] {
        match *self {
            Tag::Header => b"MThd",
            Tag::Track => b"MTrk",
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct VLQ {
    // VariableLengthQuantity
    val: u32,
}
impl VLQ {
    pub fn new(val: u32) -> VLQ {
        VLQ { val: val }
    }
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
    pub fn val(&self) -> u32 {
        self.val
    }
    fn limit_size() -> usize {
        4
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
    fn binary_192() {
        let tester = VLQ::new(192);
        assert_eq!(tester.val(), 192);
        assert_eq!(tester.len(), 2);
        assert_eq!(tester.binary(), [0x81, 0x40]);
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
pub struct VLQBuilder {
    val: u32,
    closed: bool,
}
impl VLQBuilder {
    pub fn new() -> VLQBuilder {
        VLQBuilder {
            val: 0,
            closed: false,
        }
    }
    pub fn push(&mut self, data: u8) -> &mut VLQBuilder {
        if !self.closed {
            self.val = self.val << 7 | ((data & 0b0111_1111) as u32);
        }
        if data & 0b00000000_00000000_00000000_10000000 == 0 ||
            self.val > 0b1111111_1111111_1111111
        {
            self.closed = true;
        }
        self
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn build(&self) -> VLQ {
        VLQ { val: self.val }
    }
}
