use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::frame::Opcode;

#[derive(Default)]
pub struct Message {
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn read(stream: &mut TcpStream) -> io::Result<Option<Self>> {
        let mut message = Message::default();
        loop {
            let mut header = [0, 2];
            stream.read_exact(&mut header)?;
            let fin = header[0] & 0b1000_0000 != 0;
            let opcode = Opcode::parse(&header);
            let mask = header[1] & 0b1000_0000 != 0;
            let payload_len = header[1] & 0b0111_1111;
            let payload_len = match payload_len {
                126 => {
                    let mut buf = [0; 2];
                    stream.read_exact(&mut buf)?;
                    u16::from_be_bytes(buf) as u64
                }
                127 => {
                    let mut buf = [0; 8];
                    stream.read_exact(&mut buf)?;
                    u64::from_be_bytes(buf)
                }
                len => len as u64,
            };
            let mut masking_key = None;
            if mask {
                let mut buf = [0; 4];
                stream.read_exact(&mut buf)?;
                masking_key = Some(buf);
            }
            let mut payload = vec![0; payload_len as usize];
            stream.read_exact(&mut payload)?;
            if let Some(masking_key) = masking_key {
                for (i, byte) in payload.iter_mut().enumerate() {
                    *byte ^= masking_key[i % 4];
                }
            }
            message.data.extend_from_slice(&payload);
            if fin {
                if opcode == Opcode::CLOSE {
                    return Ok(None);
                }
                break Ok(Some(message));
            }
        }
    }

    pub fn write(&self, stream: &mut TcpStream) -> io::Result<()> {
        let mut header = [0; 2];
        header[0] |= 0b1000_0000;
        header[0] |= Opcode::BINARY as u8;
        if self.data.len() > 65535 {
            header[1] |= 127;
            stream.write_all(&header)?;
            let len = self.data.len() as u64;
            stream.write_all(&len.to_be_bytes())?;
        } else if self.data.len() > 125 {
            header[1] |= 126;
            stream.write_all(&header)?;
            let len = self.data.len() as u16;
            stream.write_all(&len.to_be_bytes())?;
        } else {
            header[1] |= self.data.len() as u8;
            stream.write_all(&header)?;
        }
        stream.write_all(&self.data)?;
        stream.flush()?;
        Ok(())
    }
}
