
#[macro_export]
macro_rules! make_rpc {
    (define RPC $rpc:ident
     Global State $g:ident: $state:tt
     Control Loop: $control:tt
     Connection State $l:ident: $local:tt
     Procedures: $contract:tt) => {
        pub mod $rpc {
            use super::*;
            use $crate::Result;
            use $crate::serialization::serialize::Serialize;
            use $crate::serialization::deserialize::Deserialize;
            use $crate::serialization::transportable::Transportable;
            trait RemoteProcedure<S: Read + Write, T: Transportable<S>, Global, Local> {
                fn call(&self, stream: &mut S) -> Result<T>;
                fn handle(self, g: Arc<Global>, l: &mut Local) -> T;
                fn handle_stream(g: Arc<Global>, l: &mut Local, _stream: &mut S) -> Result<T>;
            }

            use std::sync::Arc;
            use std::thread;
            use std::io::{Read, Write};
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
            $($name : $t,)*
        }
        impl $state_name {
            fn new() -> Self {
                $state_name {$($name : $v,)*}
            }
        }

    };
    (define router {$($x:ident [$($name:ident : $param:ty),* $(as $self_:ident),*] $y:ty $implementation:block);*}) => {
        fn router<S>(g : Arc<Global>, mut stream : S) -> Result<()> where S: Read+Write {
            let mut l = Local::new();
            loop {
                let rpcname = String::decode_stream(&mut stream);
                match rpcname {
                    Ok(s) =>
                        match s.as_ref() {
                            $(stringify!($x) => {
                                let res = $x::<S>::handle_stream(g.clone(), &mut l, &mut stream);
                                if let Err(x) = res.encode_stream(&mut stream) {
                                    return Err(x)
                                }
                            },)*
                            _ =>  {let x : Result<()> = Err($crate::RPCError::NotAvailable);
                                return x.encode_stream(&mut stream)},
                        },
                    _ =>  {let x : Result<()> = Err($crate::RPCError::SerializationError);
                        return x.encode_stream(&mut stream)},
                };
            }
        }
    };
    (define rpc_loop {$($x:ident [$($name:ident : $param:ty),* $(as $self_:ident),*] $y:ty $implementation:block);*} $g:ident $control:block) => {
        fn launch_listener<A: ToSocketAddrs>(addr: A, g : Arc<Global>) -> thread::JoinHandle<()> {
            let listener = TcpListener::bind(addr).unwrap();
            thread::spawn(
                move ||
                {
                    for stream in listener.incoming() {
                        if let Ok(stream) = stream {
                            let g = g.clone();
                            thread::spawn(move || router(g, stream));
                        }
                    }
                })
        }
        pub fn main<A: ToSocketAddrs>(addr:A) {
            let mut $g = Arc::new(Global::new());
            let tcp_thread = launch_listener(addr, $g.clone());
            {
                $control
            }
            tcp_thread.join().unwrap();
        }
    };
    (define handlers $g:ident $l:ident {$($x:ident [$($name:ident : $param:ty),* $(as $self_:ident),*] $y:ty $implementation:block);*}) => {
        use std::marker::PhantomData;
        $(
            #[allow(non_camel_case_types)]
            pub struct $x<S>  where
            $($param : Transportable<S>,)*
            Result<$y> : Transportable<S>
            {
                phantom : PhantomData<S>,
                $($name : $param,)*
            }
            impl<R: Read+Write> Deserialize<R> for $x<R> {
                fn decode_stream(_s: &mut R) -> Result<$x<R>> {
                    $(
                        let $name = try!(<$param>::decode_stream(_s));
                     )*
                    let v  = $x::<R> { $($name : $name,)* phantom:PhantomData};
                    Ok(v)
                }
            }

            impl<W: Read+Write> Serialize<W> for $x<W> {
                fn encode_stream(&self, _s: &mut W) -> Result<()> {
                    $(try!(self.$name.encode_stream(_s));)*;
                    Ok(())
                }
            }
            impl<S:Read + Write> RemoteProcedure<S, $y, Global, Local> for $x<S> {
                fn call(&self, stream: &mut S)-> Result<$y>
                {
                    try!(stringify!($x).encode_stream(stream));
                    try!(self.encode_stream(stream));
                    try!(stream.flush().or_else(|_|Err($crate::RPCError::SerializationError)));
// One wrap from the deserialize, one as the result return
                    Result::<$y>::decode_stream(stream).and_then(|x| x)
                }
                fn handle(self, $g : Arc<Global>, $l : &mut Local) -> $y
                {
                    $(let $self_ = self;)*
                    $implementation
                }

                fn handle_stream($g : Arc<Global>, $l : &mut Local, _stream : &mut S) -> Result<$y>
                {
                    $x::<S>::decode_stream(_stream).and_then(|x| Ok(x.handle($g, $l)))
                }
            }
            )*
    };
    (define client {$($x:ident [$($name:ident : $param:ty),* $(as $self_:ident),*] $y:ty $implementation:block);*}) => {
        pub mod client {
            use std::net::{TcpStream,ToSocketAddrs};
            use std::{thread, time};
            pub struct Connection {
                stream : TcpStream,
            }
            use std::clone::Clone;
            pub fn new<A: ToSocketAddrs+Clone>(addr:A) -> Connection {
                for _ in 1..4 {
                    let s = TcpStream::connect(addr.clone());
                    thread::sleep(time::Duration::from_millis(10));
                    match s {
                        Ok(st) => return Connection {stream : st },
                        _ =>(),
                    }
                }
                panic!("Failed to open connection")
            }
            impl Connection{
                $(
                    pub fn $x(&mut self, $($name : $param,)*)-> $crate::Result<$y>
                    {
                        use super::RemoteProcedure;
                        use std::marker::PhantomData;
                        (super::$x::<TcpStream> {$($name:$name,)* phantom:PhantomData}).call(&mut self.stream)
                    }
                 )*
            }
        }
    }
}
