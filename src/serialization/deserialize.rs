

use super::{Result, RPCError};
use std::io;
use std::io::prelude::*;
use std::io::ErrorKind;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use byteorder::ByteOrder;
use std::vec;
use std::str;
pub trait Deserialize<R> {
    fn decode_stream(r: &mut R) -> Result<Self>
        where Self: Sized,
              R: Read;
}
impl<R: Read> Deserialize<R> for () {
    fn decode_stream(_: &mut R) -> Result<()> {
        Ok(())
    }
}

impl<R: Read> Deserialize<R> for RPCError {
    fn decode_stream(s: &mut R) -> Result<RPCError> {
        let mut buf: [u8; 1] = [0; 1];
        match s.read_exact(&mut buf) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            _ => (),
        }
        match buf[0] {
            0 => Ok(RPCError::NotAvailable),
            1 => Ok(RPCError::SerializationError),
            2 => Ok(RPCError::StreamClosed),
            3 => Ok(RPCError::UnknownError),
            _ => Err(RPCError::SerializationError),
        }

    }
}

impl<R: Read, T: Deserialize<R>> Deserialize<R> for Result<T> {
    fn decode_stream(s: &mut R) -> Result<Result<T>> {
        let mut typebuf: [u8; 1] = [0u8];
        match s.read_exact(&mut typebuf) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            _ => (),
        }
        match typebuf[0] {
            0 => {
                let t: Result<T> = Deserialize::decode_stream(s);
                match t {
                    Ok(t_) => Ok(Ok(t_)),
                    Err(x) => Err(RPCError::SerializationError),
                }
            }
            1 => {
                let t: Result<RPCError> = Deserialize::decode_stream(s);
                match t {
                    Ok(x) => Ok(Err(x)),
                    Err(x) => Err(RPCError::SerializationError),
                }
            }
            _ => Err(RPCError::SerializationError),
        }

    }
}

impl<R: Read> Deserialize<R> for u64 {
    fn decode_stream(s: &mut R) -> Result<u64> {
        let mut buf: [u8; 8] = [0; 8];
        match s.read_exact(&mut buf) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            _ => (),
        }
        Ok(BigEndian::read_u64(&buf))
    }
}
impl<R: Read> Deserialize<R> for String {
    fn decode_stream(s: &mut R) -> Result<String> {
        let size = try!(u64::decode_stream(s)) as usize;
        let mut buf = vec::Vec::new();
        buf.resize(size, 0);
        match s.read_exact(buf.as_mut_slice()) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                    _ => return Err(RPCError::UnknownError),
                }
            }
            _ => (),
        }
        match str::from_utf8(buf.as_slice()) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(RPCError::SerializationError),
        }

    }
}
