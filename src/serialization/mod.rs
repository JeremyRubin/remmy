extern crate byteorder;
pub use super::{Result, RPCError};
pub mod serialize;
pub mod deserialize;
pub mod transportable;



#[cfg(test)]
mod serialization_tests {
    use super::*;
    use serialization::serialize::Serialize;
    use serialization::deserialize::Deserialize;
    #[test]
    fn serdeser_u64() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(8, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: u64 = 100;
        {
            a.encode_stream(&mut buff);
        }
        buff.set_position(0);
        let x: Result<u64> = u64::decode_stream(&mut buff);
        match x {
            Ok(x) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_string() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(8, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: String = "12345678".to_string();
        {
            a.encode_stream(&mut buff);
        }
        buff.set_position(0);
        let x: Result<String> = String::decode_stream(&mut buff);
        match x {
            Ok(x) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_result_ok_string() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(100, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: String = "1234567".to_string();
        {
            let b = a.clone();
            Ok(b).encode_stream(&mut buff);
        }
        buff.set_position(0);
        let x: Result<Result<String>> = Result::<String>::decode_stream(&mut buff);
        match x {
            Ok(Ok(x)) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_result_err() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(100, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let r: Result<String> = Err(RPCError::SerializationError);
        r.encode_stream(&mut buff);
        buff.set_position(0);
        let x: Result<Result<String>> = Result::<String>::decode_stream(&mut buff);
        match x {
            Ok(Err(RPCError::SerializationError)) => (),
            _ => panic!("Failed to deserialize Err(SerializationError) properly"),
        }
    }
}
