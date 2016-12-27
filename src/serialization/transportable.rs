use super::{Serialize, Deserialize, RPCError, Result};
use std::io;
use std::io::prelude::*;
pub trait Transportable<S>: Serialize<S> + Deserialize<S> {}
macro_rules! d {
    ($t:ty) => {
        impl<S: Read + Write> Transportable<S> for $t {}
    }
}
d!(());
d!(RPCError);
d!(u64);
d!(String);
impl<S: Read + Write, T: Transportable<S>> Transportable<S> for Result<T> {}