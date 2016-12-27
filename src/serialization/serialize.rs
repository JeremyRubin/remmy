use super::{Result, RPCError};
use std::io;
use std::io::prelude::*;
use std::io::ErrorKind;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use byteorder::ByteOrder;

pub trait Serialize<W> {
    fn encode_stream(&self, stream: &mut W) -> Result<()> where W: Write;
}

impl<W: Write> Serialize<W> for String {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        (self.len() as u64).encode_stream(s);
        s.write_all(self.as_bytes());
        Ok(())
    }
}
impl<W: Write> Serialize<W> for u64 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let mut buf: [u8; 8] = [0; 8];
        BigEndian::write_u64(&mut buf, *self);
        match s.write_all(&buf) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            Ok(_) => return Ok(()),
        }
    }
}

impl<W: Write> Serialize<W> for () {
    fn encode_stream(&self, _: &mut W) -> Result<()> {
        Ok(())
    }
}
impl<W: Write> Serialize<W> for RPCError {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let buf: [u8; 1] = [match *self {
                                RPCError::NotAvailable => 0,
                                RPCError::SerializationError => 1,
                                RPCError::StreamClosed => 2,
                                RPCError::UnknownError => 3,
                            }];
        match s.write_all(&buf) {
            Ok(_) => Ok(()),
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }

        }
    }
}

impl<W: Write, T: Serialize<W>> Serialize<W> for Result<T> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let r = match self {
            &Ok(ref a) => {
                let x = s.write_all(&[0]);
                let y = a.encode_stream(s);
                (x, y)
            }
            &Err(ref a) => {
                let x = s.write_all(&[1]);
                let y = a.encode_stream(s);
                (x, y)
            }
        };
        match r {
            (Err(e), _) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            (_, Err(e)) => Err(e),
            (Ok(_), Ok(_)) => return Ok(()),
        }
    }
}
