pub mod cdr_bounds;
mod duration;
mod error;
pub mod state;
pub mod status;
mod time;
mod topicable;

pub use cyclonedds_macros::Topicable;
pub use duration::Duration;
pub use error::{Error, Result};
pub use state::State;
pub use status::bitflags::Status;
pub use time::Time;
pub use topicable::{Key, Topicable};

#[cfg(feature = "internal")]
pub mod internal;
#[cfg(not(feature = "internal"))]
mod internal;

#[cfg(test)]
mod tests;
