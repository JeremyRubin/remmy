extern crate byteorder;



#[derive(Debug)]
pub enum RPCError {
    NotAvailable,
    SerializationError,
    StreamClosed,
}
use std::result;
pub type Result<T> = result::Result<T, RPCError>;

pub mod serialization;
pub use serialization::*;

#[macro_use]
mod rpc_macro;

#[cfg(test)]
mod serialization_tests {
    pub use super::*;
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
        let x: Result<u64> = Deserialize::decode_stream(&mut buff);
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
        let x: Result<String> = Deserialize::decode_stream(&mut buff);
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
        let x: Result<Result<String>> = Deserialize::decode_stream(&mut buff);
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
        let x: Result<Result<String>> = Deserialize::decode_stream(&mut buff);
        match x {
            Ok(Err(RPCError::SerializationError)) => (),
            _ => panic!("Failed to deserialize Err(SerializationError) properly"),
        }
    }
}

#[cfg(test)]
mod rpc_tests {
    use std::{time, thread};
    pub use super::*;
    make_rpc!(define RPC server
              Global State: {
                  counter : i64
              }
              Local State: {
              }
              Functions: {
                  increment_by(amount:u64) -> u64{Ok(amount)};
                  decrement() -> u64{Ok(1) }
              }
             );
    #[test]
    fn test() {
        thread::spawn(|| server::rpc_loop("localhost:8080"));
        thread::sleep(time::Duration::from_secs(1));
        let res = server::client::new("localhost:8080")
                      .increment_by(100)
                      .unwrap();

        thread::sleep(time::Duration::from_secs(1));
    }
}
