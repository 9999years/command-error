use std::fmt::Debug;
use std::fmt::Display;

/// A type that implements [`Debug`] and [`Display`].
///
/// Used to store formattable and debuggable objects in [`Box<dyn DebugDisplay>`] containers.
pub trait DebugDisplay: Debug + Display {}

impl<T> DebugDisplay for T where T: Debug + Display {}
