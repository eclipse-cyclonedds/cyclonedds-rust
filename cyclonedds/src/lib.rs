pub mod cdr_bounds;
mod domain;
mod duration;
pub mod entity;
mod error;
mod guard_condition;
mod participant;
mod publisher;
pub mod qos;
mod query_condition;
mod read_condition;
mod reader;
pub mod sample;
pub mod state;
pub mod status;
mod subscriber;
mod time;
mod topic;
mod topicable;
mod waitset;
mod writer;

pub use cyclonedds_macros::Topicable;
pub use domain::Domain;
pub use duration::Duration;
pub use error::{Error, Result};
pub use guard_condition::GuardCondition;
pub use participant::Participant;
pub use publisher::Publisher;
pub use qos::QoS;
pub use query_condition::QueryCondition;
pub use read_condition::ReadCondition;
pub use reader::Reader;
pub use state::State;
pub use status::bitflags::Status;
pub use subscriber::Subscriber;
pub use time::Time;
pub use topic::Topic;
pub use topicable::{Key, Topicable};
pub use waitset::WaitSet;
pub use writer::Writer;

pub mod builder {
    //! Builder types for constructing DDS entities with custom `QoS` and
    //! listeners.
    //!
    //! Each builder is also accessible via the `builder` associated function on
    //! its corresponding entity type.
    pub use crate::participant::ParticipantBuilder;
    pub use crate::publisher::PublisherBuilder;
    pub use crate::reader::ReaderBuilder;
    pub use crate::subscriber::SubscriberBuilder;
    pub use crate::topic::TopicBuilder;
    pub use crate::writer::WriterBuilder;
}

#[cfg(feature = "internal")]
pub mod internal;
#[cfg(not(feature = "internal"))]
mod internal;

#[cfg(test)]
mod tests;
