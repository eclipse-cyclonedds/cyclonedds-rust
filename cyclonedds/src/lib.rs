pub mod cdr_bounds;
mod error;

pub use error::{Error, Result};

#[cfg(feature = "internal")]
pub mod internal;
#[cfg(not(feature = "internal"))]
mod internal;

#[cfg(test)]
mod tests;
