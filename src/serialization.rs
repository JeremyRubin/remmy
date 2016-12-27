extern crate byteorder;
use std::io;
use std::io::prelude::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use byteorder::ByteOrder;
use std::result;
use std::vec;
use std::str;
use super::RPCError;
use super::Result;
pub trait Serialize<W> {
    fn encode_stream(&self, stream: &mut W) -> Result<()> where W: Write;
}
pub trait Deserialize<R> {
    fn decode_stream(r: &mut R) -> Result<Self>
        where Self: Sized,
              R: Read;
}
pub trait Transportable<S>: Serialize<S> + Deserialize<S> {}



impl<R: Read> Deserialize<R> for () {
    fn decode_stream(s: &mut R) -> Result<()> {
        Ok(())
    }
}
impl<W: Write> Serialize<W> for () {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        Ok(())
    }
}
impl<S: Read + Write> Transportable<S> for () {}

impl<R: Read> Deserialize<R> for RPCError {
    fn decode_stream(s: &mut R) -> Result<RPCError> {
        let mut buf: [u8; 1] = [0; 1];
        match s.read_exact(&mut buf) {
            Err(UnexpectedEof) => return Err(RPCError::StreamClosed),
            _ => (),
        }
        match buf[0] {
            0 => Ok(RPCError::NotAvailable),
            1 => Ok(RPCError::SerializationError),
            2 => Ok(RPCError::StreamClosed),
            _ => Err(RPCError::SerializationError),
        }

    }
}
impl<W: Write> Serialize<W> for RPCError {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let mut buf: [u8; 1] = [match *self {
                                    RPCError::NotAvailable => 0,
                                    RPCError::SerializationError => 1,
                                    RPCError::StreamClosed => 2,
                                }];
        s.write_all(&buf);
        Ok(())
    }
}
impl<S: Read + Write> Transportable<S> for RPCError {}


impl<R: Read, T: Deserialize<R>> Deserialize<R> for Result<T> {
    fn decode_stream(s: &mut R) -> Result<Result<T>> {
        let mut buf = [0];
        match s.read_exact(&mut buf) {
            Err(UnexpectedEof) => return Err(RPCError::StreamClosed),
            _ => (),
        }
        match buf[0] {
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
            (Err(UnexpectedEof), _) => return Err(RPCError::StreamClosed),
            (_, Err(UnexpectedEof)) => return Err(RPCError::StreamClosed),
            (Ok(_), Ok(_)) => return Ok(()),
        }
    }
}
impl<S: Read + Write, T: Transportable<S>> Transportable<S> for Result<T> {}

impl<R: Read> Deserialize<R> for u64 {
    fn decode_stream(s: &mut R) -> Result<u64> {
        let mut buf: [u8; 8] = [0; 8];
        match s.read_exact(&mut buf) {
            Err(UnexpectedEof) => return Err(RPCError::StreamClosed),
            _ => (),
        }
        Ok(BigEndian::read_u64(&buf))
    }
}
impl<W: Write> Serialize<W> for u64 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let mut buf: [u8; 8] = [0; 8];
        BigEndian::write_u64(&mut buf, *self);
        match s.write_all(&buf) {
            Err(UnexpectedEof) => return Err(RPCError::StreamClosed),
            Ok(_) => return Ok(()),
        }
    }
}
impl<S: Read + Write> Transportable<S> for u64 {}

impl<R: Read> Deserialize<R> for String {
    fn decode_stream(s: &mut R) -> Result<String> {
        let size = try!(u64::decode_stream(s)) as usize;
        let mut buf = vec::Vec::new();
        buf.resize(size, 0);
        match s.read_exact(buf.as_mut_slice()) {
            Err(UnexpectedEof) => return Err(RPCError::StreamClosed),
            _ => (),
        }
        match str::from_utf8(buf.as_slice()) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(RPCError::SerializationError),
        }

    }
}
impl<W: Write> Serialize<W> for String {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        (self.len() as u64).encode_stream(s);
        s.write_all(self.as_bytes());
        Ok(())
    }
}
impl<S: Read + Write> Transportable<S> for String {}
