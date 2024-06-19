#[derive(PartialEq, Eq)]
pub enum Opcode {
    CONTINUE = 0x0,
    TEXT = 0x1,
    BINARY = 0x2,
    CLOSE = 0x8,
    PING = 0x9,
    PONG = 0xA,
}

impl Opcode {
    pub fn parse(header: &[u8; 2]) -> Self {
        match header[0] & 0b0000_1111 {
            0x0 => Self::CONTINUE,
            0x1 => Self::TEXT,
            0x2 => Self::BINARY,
            0x8 => Self::CLOSE,
            0x9 => Self::PING,
            0xA => Self::PONG,
            _ => unimplemented!(),
        }
    }
}
