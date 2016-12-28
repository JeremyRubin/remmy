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
mod rpc_tests {
    use std::{time, thread};
    pub use super::*;
    pub use std::cell::Cell;

    pub use std::sync::Mutex;
    pub use std::ops::DerefMut;
    make_rpc!(define RPC server
              Global State g: {
                  let counter : Mutex<u64> = Mutex::new(0);
                  let counter2 : i64 = 0
              }
              Connection State l: {
                  let cache : String = String::new()

              }
              Procedures: {
                  echo(a:u64) -> u64{a};
                  increment() -> u64{
                      let mut data = g.counter.lock().unwrap();
                      *data += 1;
                      *data
                  };
                  decrement() -> u64{
                      let mut data = g.counter.lock().unwrap();
                      *data -= 1;
                      *data
                  };
                  cache(s:String) -> u64 {
                      l.cache.clear();
                      l.cache.push_str(s.as_str());
                      1
                  };
                  fetch_cache() -> String {
                      l.cache.clone()
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
