extern crate byteorder;



#[derive(Debug)]
pub enum RPCError {
    NotAvailable,
    SerializationError,
    StreamClosed,
    UnknownError,
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
    pub use std::cell::Cell;

    pub use std::sync::Mutex;
    pub use std::ops::DerefMut;
    make_rpc!(define RPC server
              Global State G: {
                  let counter : Mutex<u64> = Mutex::new(0);
                  let counter2 : i64 = 0
              }
              Connection State L: {
                  let cache : String = String::new()

              }
              Procedures: {
                  echo(a:u64) -> u64{a};
                  increment() -> u64{
                      let mut data = G.counter.lock().unwrap();
                      *data += 1;
                      *data
                  };
                  decrement() -> u64{
                      let mut data = G.counter.lock().unwrap();
                      *data -= 1;
                      *data
                  };
                  cache(s:String) -> u64 {
                      L.cache.clear();
                      L.cache.push_str(s.as_str());
                      1
                  };
                  fetch_cache() -> String {
                      L.cache.clone()
                  }
              }
             );
    #[test]
    fn test_rpc() {
        println!("Spawning");
        let th = thread::spawn(|| server::rpc_loop("localhost:8000"));
        println!("Spawned");
        {
            let mut conn = server::client::new("localhost:8000");
            println!("Got Connection");
            for i in 1..100 {
                assert_eq!(conn.echo(i).unwrap(), i);
            }

            for i in 1..101 {
                let j = conn.increment().unwrap();
                println!("{} == {}", i, j);
                assert_eq!(i, i);
            }
            for i in (0..100).rev() {
                let j = conn.decrement().unwrap();
                println!("{} == {}", i, j);
                assert_eq!(i, i);
            }
            conn.cache("hello".to_string());
            assert_eq!(conn.fetch_cache().unwrap(), "hello".to_string());
            println!("Got: {}", conn.fetch_cache().unwrap());

        }
    }
}
