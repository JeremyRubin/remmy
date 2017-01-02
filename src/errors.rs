use std::io::*;
use slim;
#[derive(Debug)]
pub enum RPCError {
    NotAvailable,
    SerializationError(slim::SlimError),
}

use std::result;
impl<R: Read> slim::Deserialize<R> for RPCError {
    fn decode_stream(s: &mut R) -> result::Result<RPCError, slim::SlimError> {
        let x = try!(u8::decode_stream(s));
        match x {
            0 => Ok(RPCError::NotAvailable),
            1 => Ok(RPCError::SerializationError(try!(slim::SlimError::decode_stream(s)))),
            _ => Err(slim::SlimError::DeserializationError),
        }

    }
}
impl<W: Write> slim::Serialize<W> for RPCError {
    fn encode_stream(&self, s: &mut W) -> result::Result<(), slim::SlimError> {
        match *self {
            RPCError::NotAvailable => 0u8.encode_stream(s),
            RPCError::SerializationError(ref x) => {
                try!(1u8.encode_stream(s));
                x.encode_stream(s)
            }

        }
    }
}
impl<S: Read + Write> slim::Transportable<S> for RPCError {}
