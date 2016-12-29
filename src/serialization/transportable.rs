use super::serialize::Serialize;
use super::deserialize::Deserialize;
use super::{RPCError, Result};
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
impl<S: Read + Write, T: Transportable<S>> Transportable<S> for Option<T> {}
use std::borrow::Cow;
impl<'a, S: Read + Write, T: Transportable<S> + Clone> Transportable<S> for Cow<'a, T> {}
