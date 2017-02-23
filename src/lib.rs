// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
extern crate slim;

pub use slim::{Deserialize, Serialize, Transportable};


use std::result;
pub type Result<T> = result::Result<T, RPCError>;

pub mod errors;
pub use errors::*;


#[macro_use]
pub mod rpc_macro;

#[cfg(test)]
mod rpc_tests {
    use std::thread;
    pub use std::sync::Mutex;
    make_rpc!(define RPC server
              Global State _g: {
                  let counter : Mutex<u64> = Mutex::new(0);
                  let alive : Mutex<u8> = Mutex::new(0)
              }
              Control Loop: {
                  use std::time;
                  while *_g.alive.lock().unwrap() == 0 {
                      thread::sleep(time::Duration::from_secs(1));
                  }
                  *_g.alive.lock().unwrap() = 2;
              }
              Connection State _l: {
                  let cache : String = String::new()
              }
              Procedures: {
                  echo [a:u64 as msg] u64{msg.a};
                  increment [] u64{
                      let mut data = _g.counter.lock().unwrap();
                      *data += 1;
                      *data
                  };
                  decrement []  u64{
                      let mut data = _g.counter.lock().unwrap();
                      *data -= 1;
                      *data
                  };
                  cache [s:String as msg]  u64 {
                      _l.cache.clear();
                      _l.cache.push_str(msg.s.as_str());
                      1
                  };
                  fetch_cache []  String {
                      _l.cache.clone()
                  };
                  shutdown []  () {
                      {
                          let mut x = _g.alive.lock().unwrap();
                          match *x {
                              0 => *x = 1,
                              1 => return (),
                              _ => return (),
                          }
                      }
                      {
                          loop {
                              use std::time;
                              thread::sleep(time::Duration::from_millis(100));
                              if *(_g.alive.lock().unwrap()) == 2 {
                                  return ()
                              }
                          }
                      }
                  }
              });


    #[test]
    fn test_rpc() {
        let _ = thread::spawn(|| server::main("localhost:8000"));
        {
            let mut conn = server::client::new("localhost:8000");
            for i in 1..100 {
                assert_eq!(conn.echo(i).unwrap(), i);
            }

            for i in 1..101 {
                let j = conn.increment().unwrap();
                assert_eq!(i, j);
            }
            for i in (0..100).rev() {
                let j = conn.decrement().unwrap();
                assert_eq!(i, j);
            }
            conn.cache("hello".to_string()).unwrap();
            assert_eq!(conn.fetch_cache().unwrap(), "hello".to_string());
            conn.shutdown().unwrap();
        }

    }
}
