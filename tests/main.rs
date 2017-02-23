// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate remmy;
extern crate slim;

pub use std::sync::Mutex;
pub use std::collections::hash_map::HashMap;
pub use std::collections::vec_deque;
pub use std::sync::RwLock;
pub use std::borrow::Cow;
pub type Table = RwLock<vec_deque::VecDeque<String>>;
make_rpc!(define RPC link_shortener
          Global State _g: {
              let table : Table = Table::new(vec_deque::VecDeque::new());
              let counter : RwLock<usize> = RwLock::new(0);
              let alive : Mutex<u8> = Mutex::new(0)
          }
          Control Loop: {
              while *(_g.alive.lock().unwrap()) == 0 {
                  let l1 = _g.table.write().unwrap().len();
                  use std::time;
                  thread::sleep(time::Duration::from_millis(100));
                  let mut data = _g.table.write().unwrap();
                  let mut counter = _g.counter.write().unwrap();
                  for _ in 0..l1 {
                    data.pop_front();
                  }
                  *counter += l1;

              }
              *_g.alive.lock().unwrap() = 2;
          }
          Connection State _l: {
          }
          Procedures: {
              fetch_link [s : u64 as msg] Option<String>{
                  let data = _g.table.read().unwrap();
                  let counter = _g.counter.read().unwrap();
                  if (msg.s as usize) >= *counter  {
                     return data.get((msg.s as usize) - *counter).cloned()
                  }
                  None
              };
              make_link [s : String as msg] u64 {
                  let mut data = _g.table.write().unwrap();
                  let counter = _g.counter.read().unwrap();
                  data.push_back(msg.s);
                  (data.len()-1+ *counter) as u64
              };
              shutdown [] () {
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
          }
);
#[cfg(test)]
mod integration_test {
    use std::thread;
    use super::*;
    use std::sync::atomic;
    static mut m: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;
    #[test]
    fn test_ttl() {
        unsafe {
            while m.swap(true, atomic::Ordering::SeqCst) {
            }
        }
        let _ = thread::spawn(|| link_shortener::main("localhost:8000"));
        let mut conn = link_shortener::client::new("localhost:8000");
        {
            for i in 0..100 {
                let x: u64 = conn.make_link(format!("{}", i)).unwrap();
                assert_eq!(conn.fetch_link(x).unwrap(), Some(i.to_string()));
            }
            use std::time;
            thread::sleep(time::Duration::from_millis(200));
            for i in 0..100 {
                assert_eq!(None, conn.fetch_link(i).unwrap());
            }
            for i in 0..100 {
                let x: u64 = conn.make_link(format!("{}", i)).unwrap();
                assert_eq!(conn.fetch_link(x).unwrap(), Some(i.to_string()));
            }
        }
        conn.shutdown().unwrap();
        unsafe {
            m.swap(false, atomic::Ordering::SeqCst);
        }
    }
    #[test]
    fn test_many_clients() {
        unsafe {
            while m.swap(true, atomic::Ordering::SeqCst) {
            }
        }
        let _ = thread::spawn(|| link_shortener::main("localhost:8000"));
        use std::vec;
        let s: vec::Vec<thread::JoinHandle<()>> =
            (0..100)
                .map(|_: u64| {
                    thread::spawn(move || {
                        let mut conn = link_shortener::client::new("localhost:8000");
                        for i in 0..100 {
                            let x: u64 = conn.make_link(format!("{}", i)).unwrap();
                            assert_eq!(conn.fetch_link(x).unwrap(), Some(i.to_string()));
                        }
                    })
                })
                .collect();
        for thr in s {
            match thr.join() {
                Err(_) => assert!(false),
                Ok(()) => (),

            }
        }
        link_shortener::client::new("localhost:8000").shutdown().unwrap();
        unsafe {
            m.swap(false, atomic::Ordering::SeqCst);
        }
    }

}
