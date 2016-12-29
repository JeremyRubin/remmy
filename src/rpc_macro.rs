use std::net::TcpStream;
use std::io::{BufReader, BufWriter, Read, Write};
#[macro_export]
macro_rules! make_rpc {
    (define RPC $rpc:ident
     Global State $g:ident: $state:tt
     Control Loop: $control:tt
     Connection State $l:ident: $local:tt
     Procedures: $contract:tt) => {
        pub mod $rpc {
            use super::*;
            pub use $crate::RPCError;
            pub use $crate::Result;
            pub use $crate::serialization::serialize::Serialize;
            pub use $crate::serialization::deserialize::Deserialize;
            pub use $crate::serialization::transportable::Transportable;

            pub use std::sync::Arc;
            pub use std::thread;
            pub use std::io::{Read, Write};
            make_rpc!(define state Global $state);
            make_rpc!(define state Local $local);
            make_rpc!(define handlers $g $l $contract);
            use std::net::{TcpListener, ToSocketAddrs};
            make_rpc!(define router $contract);
            make_rpc!(define rpc_loop $contract $g $control);
            make_rpc!(define client $contract);
        }
    };
    (define state $state_name:ident {$(let $name:ident : $t:ty =  $v:expr);*}) => {
        pub struct $state_name {
            should_quit : bool,
            $($name : $t,)*
        }
        impl $state_name {
            fn new() -> Self {
                $state_name {should_quit : false, $($name : $v,)*}
            }
        }

    };
    (define router {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        fn router<S>(G : Arc<Global>, mut stream : S) -> Result<()> where S: Read+Write {
            let mut L = Local::new();
            loop {
                let rpcname = String::decode_stream(&mut stream);
                match rpcname {
                    Ok(s) =>
                        match s.as_ref() {
                            $(stringify!($x) => {
                                let res = $x::handle_stream(G.clone(), &mut L, &mut stream);
                                if let Err(x) = res.encode_stream(&mut stream) {
                                    return Err(x)
                                }
                            },)*
                            _ =>  {let x : Result<()> = Err(RPCError::NotAvailable);
                                return x.encode_stream(&mut stream)},
                        },
                    _ =>  {let x : Result<()> = Err(RPCError::SerializationError);
                        return x.encode_stream(&mut stream)},
                };
            }
        }
    };
    (define rpc_loop {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*} $g:ident $control:block) => {
        fn launch_listener<A: ToSocketAddrs>(addr: A, G : Arc<Global>) -> thread::JoinHandle<()> {
            let listener = TcpListener::bind(addr).unwrap();
            thread::spawn(
                move ||
                {
                    for stream in listener.incoming() {
                        if let Ok(stream) = stream {
                            let G = G.clone();
                            thread::spawn(move || router(G, stream));
                        }
                    }
                })
        }
        pub fn rpc_loop<A: ToSocketAddrs>(addr:A) {
            let mut $g = Arc::new(Global::new());
            let tcp_thread = launch_listener(addr, $g.clone());
            $control
            tcp_thread.join();
        }
    };
    (define handlers $g:ident $l:ident {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        $(
            pub mod $x {
                use super::*;
                pub fn call<S>(stream: &mut S, $($name : $param,)*)-> Result<$y>
                    where
                    S : Read + Write,
                    $($param : Transportable<S>,)*
                    $y:Transportable<S>
                    {
                        let rpcname = stringify!($x).to_string();
                        try!(rpcname.encode_stream(stream));
                        $(try!($name.encode_stream(stream));)*;
                        stream.flush();
// One wrap from the deserialize, one as the result return
                        let response = Result::<$y>::decode_stream(stream);
                        match response {
                            Ok(x) => x,
                            Err(x) => Err(x),
                        }
                    }
                fn handle($g : Arc<Global>, $l : &mut Local, $($name : $param,)*) -> $y
                {
                    $implementation
                }

                pub fn handle_stream<R:Read>($g : Arc<Global>, $l : &mut Local, stream : &mut R) -> Result<$y>
                    where
                    $( $param : Transportable<R>,)*
                    $y:Transportable<R>
                    {
                        $(
                            let $name = <$param>::decode_stream(stream);
                            let $name = match $name {
                                Ok(v) => v,
                                Err(x) =>{
                                    let e = Err(x);
                                    return e
                                },
                            };
                         )*
                           Ok(handle($g, $l, $($name,)*))
                    }
            }
        )*
    };
    (define client {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        pub mod client {
            use super::*;
            use std::net::{TcpStream,ToSocketAddrs};
            pub struct Connection {
                stream : TcpStream,
            }
                use std::clone::Clone;
            pub fn new<A: ToSocketAddrs+Clone>(addr:A) -> Connection {
                use std::time;
                for i in 1..4 {
                    let s = TcpStream::connect(addr.clone());
                    thread::sleep(time::Duration::from_millis(10));
                    match s {
                        Ok(st) => return Connection {stream : st },
                        _ =>(),
                    }
                }
                panic!("Failed to open connection")
            }
            impl Connection {
                $(
                    pub fn $x(&mut self, $($name : $param,)*)-> Result<$y>
                    where
                    $( $param : Transportable<TcpStream>,)*
                    $y:Transportable<TcpStream>
                    {
                        $x::call(&mut self.stream, $($name,)*)
                    }
                 )*
            }
        }
    }
}
