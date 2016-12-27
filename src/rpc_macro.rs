
use std::net::TcpStream;
macro_rules! make_rpc {
    (define RPC $rpc:ident
     Global State: $state:tt
     Local State: $local:tt
     Functions: $contract:tt) => {
        mod $rpc {
            pub use std::io::prelude::*;
            pub use std::net::{TcpStream,TcpListener, ToSocketAddrs};
            pub use super::*;
            make_rpc!(define state global $state);
            make_rpc!(define state local $local);
            make_rpc!(define handlers $contract);
            make_rpc!(define router $contract);
            make_rpc!(define rpc_loop $contract);
            make_rpc!(define client $contract);
        }
    };
    (define state $state_name:ident {$($name:ident : $t:ty),*}) => {
        pub struct $state_name {
            should_quit : bool,
            $(
                $name : $t,
                )*
        }
    };
    (define router {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        fn router(mut stream : TcpStream) -> Result<()> {
            loop {
                let x : String = try!(Deserialize::decode_stream(&mut stream));
                try!(match x.as_ref() {
                    $(
                        stringify!($x) => $x::handle_stream(&mut stream),
                        )*
                        _ =>  {let x : Result<()> = Err(RPCError::NotAvailable);
                            return x.encode_stream(&mut stream)},
                })
            }
        }
    };
    (define rpc_loop {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        pub fn rpc_loop<A: ToSocketAddrs>(addr:A) {
            let listener = TcpListener::bind(addr).unwrap();
// accept connections and process them, spawning a new thread for each one
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        router(stream);
                        ()
                    }
                    Err(e) => {
// connection failed
                    }
                }
            }
        }
    };
    (define handlers {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        $(
            pub mod $x {
                use super::*;
                pub fn call(stream: &mut TcpStream, $($name : $param,)*)-> Result<$y>
                    where
                    $( $param : Transportable<TcpStream>,)*
                    $y:Transportable<TcpStream>
                    {
                        {
                            (stringify!($x)).to_string().encode_stream(stream);
                        }
                        $(
                            {
                                $name.encode_stream(stream);
                            }
                         )*
                            let res : $y = try!(Deserialize::decode_stream(stream));
                        Ok(res)
                    }
                fn handle($($name : $param,)*)-> Result<$y>
                    where
                        $( $param : Transportable<TcpStream>,)*
                        $y:Transportable<TcpStream>
                        {
                            $implementation
                        }

                pub fn handle_stream(stream : &mut TcpStream) -> Result<()>
                    where
                    $( $param : Transportable<TcpStream>,)*
                    $y:Transportable<TcpStream>
                    {
                        $(
                            let $name : $param = try!(Deserialize::decode_stream(stream));
                         )*
                           handle($( $name, )*).encode_stream(stream)
                    }
            }
        )*
    };
    (define client {$($x:ident($($name:ident : $param:ty),*) -> $y:ty $implementation:block);*}) => {
        pub mod client {
            use super::*;
            pub struct connection {
                stream : TcpStream,
            }

            pub fn new<A: ToSocketAddrs>(addr:A) -> connection {
                connection {stream : TcpStream::connect(addr).unwrap() }
            }
            impl connection {

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
