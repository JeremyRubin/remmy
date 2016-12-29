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
fn write_buf<W: Write>(s: &mut W, buf: &[u8]) -> Result<()> {

    match s.write_all(buf) {
        Err(e) => {
            match e.kind() {
                ErrorKind::UnexpectedEof => return Err(RPCError::StreamClosed),
                _ => return Err(RPCError::UnknownError),
            }
        }
        Ok(_) => return Ok(()),
    }
}
macro_rules! ser_int {
    ($a:ty, $b:expr, $c:path)=>    {
        impl<W: Write> Serialize<W> for $a {
            fn encode_stream(&self, s: &mut W) -> Result<()> {
                let mut buf: [u8; $b] = [0; $b];
                $c(&mut buf, *self);
                write_buf(s, &buf)
            }
        }
    }
}

ser_int!(u64, 8, BigEndian::write_u64);
ser_int!(u32, 4, BigEndian::write_u32);
ser_int!(u16, 2, BigEndian::write_u16);
ser_int!(i64, 8, BigEndian::write_i64);
ser_int!(i32, 4, BigEndian::write_i32);
ser_int!(i16, 2, BigEndian::write_i16);
ser_int!(f64, 8, BigEndian::write_f64);
ser_int!(f32, 4, BigEndian::write_f32);

impl<W: Write> Serialize<W> for u8 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self])
    }
}
impl<W: Write> Serialize<W> for i8 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self as u8])
    }
}
impl<W: Write> Serialize<W> for bool {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self as u8])
    }
}
impl<W: Write> Serialize<W> for () {
    fn encode_stream(&self, _: &mut W) -> Result<()> {
        Ok(())
    }
}
impl<W: Write> Serialize<W> for RPCError {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let buf: u8 = match *self {
            RPCError::NotAvailable => 0,
            RPCError::SerializationError => 1,
            RPCError::StreamClosed => 2,
            RPCError::UnknownError => 3,
        };
        buf.encode_stream(s)
    }
}

impl<W: Write, T: Serialize<W>> Serialize<W> for Result<T> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        match self {
            &Ok(ref a) => {
                try!(0u8.encode_stream(s));
                a.encode_stream(s)
            }
            &Err(ref a) => {
                try!(1u8.encode_stream(s));
                a.encode_stream(s)
            }
        }
    }
}

impl<W: Write, T: Serialize<W>> Serialize<W> for Option<T> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        match self {
            &None => 0u8.encode_stream(s),
            &Some(ref a) => {
                try!(1u8.encode_stream(s));
                a.encode_stream(s)
            }
        }
    }
}
