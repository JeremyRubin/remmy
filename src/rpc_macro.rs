
use std::net::TcpStream;
use std::io::{BufReader, BufWriter};
macro_rules! make_rpc {
    (define RPC $rpc:ident
     Global State $g:ident: $state:tt
     Connection State $l:ident: $local:tt
     Procedures: $contract:tt) => {
        pub mod $rpc {
            pub use std::io::prelude::*;
            pub use super::*;
            pub use std::sync::Arc;
            pub use std::thread;
            make_rpc!(define state Global $state);
            make_rpc!(define state Local $local);
            make_rpc!(define handlers $g $l $contract);
            use std::net::{TcpListener, ToSocketAddrs};
            make_rpc!(define router $contract);
            make_rpc!(define rpc_loop $contract);
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
                let rpcname : Result<String> = Deserialize::decode_stream(&mut stream);
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
    (define rpc_loop {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        pub fn rpc_loop<A: ToSocketAddrs>(addr:A) {
            let mut G = Arc::new(Global::new());
            let listener = TcpListener::bind(addr);
            if let Ok(l) = listener {
                for stream in l.incoming() {
                    if let Ok(stream) = stream {
                        let G = G.clone();
                        thread::spawn(move || router(G, stream));
                        ()
                    }
                }
            }
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
                        let response : Result<Result<$y>> = Deserialize::decode_stream(stream);
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
                            let $name : $param = match Deserialize::decode_stream(stream) {
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
            pub fn new<A: ToSocketAddrs>(addr:A) -> Connection {
                let s = TcpStream::connect(addr);
                match s {
                    Ok(st) => Connection {stream : st },
                    _ => panic!("failed to open connection"),
                }
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
