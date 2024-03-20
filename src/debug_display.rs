use std::fmt::Debug;
use std::fmt::Display;

pub(crate) trait DebugDisplay: Debug + Display {}

impl<T> DebugDisplay for T where T: Debug + Display {}
