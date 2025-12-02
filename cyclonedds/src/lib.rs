//! The official Rust binding for
//! [Cyclone DDS](https://github.com/eclipse-cyclonedds/cyclonedds).
//!
//! DDS (Data Distribution Service) is a publish-subscribe middleware standard
//! for real-time, data-centric communication. It is used in a variety of
//! mission critical applications in domains such as aerospace, defense,
//! autonomous systems (e.g. vehicles, robotics), industrial control, smart
//! energy grids, transportation, simulation, and medical devices.
//!
//! [`Participants`](Participant) within a specific [`Domain`] discover each
//! other automatically via the DDSI/RTPS discovery protocol. Once two endpoints
//! sharing the same topic name, type information, and compatible
//! [`Quality of Service`][QoS] (`QoS`) discover each other, the middleware
//! establishes a connection between them.
//!
//! [`Publishers`](Publisher) and [`Subscribers`](Subscriber) allow you to group
//! [`Writers`](Writer) and [`Readers`](Reader) respectively to allow you to set
//! their collective behavior. These [`Writers`](Writer) and [`Readers`](Reader)
//! exchange typed samples via [`Topics`](Topic).
//!
//! ```text
//!                             DOMAIN
//!                                │
//!             ┌──────────────────┴──────────────────┐
//!             │                                     │
//!        PARTICIPANT                           PARTICIPANT
//!             │      T ≡ struct Position {x, y}     │
//!        ┌────┴────┐                           ┌────┴────┐
//!        │         │                           │         │
//!   PUBLISHER   TOPIC<T> ═══════════════════ TOPIC<T>  SUBSCRIBER
//!        │         ║                           ║         │
//!        │     "Position"                 "Position"     │
//!        │         ║                           ║         │
//!     WRITER<T> ═══╝                           ╚═══ READER<T>
//!          ╰───────── matched via Topic<T> ─────────╯
//!          Node 01                               Node 02
//!         ─────────                             ─────────
//! ```
//!
//! Data delivery characteristics, such as how samples are buffered,
//! retransmitted, and received, are controlled via [`Quality of Service`][QoS],
//! a collection of
//! [`QoS policies`](qos::policy) that configure characteristics such as:
//!
//! - [`durability`](qos::policy::Durability) (whether late-joining readers
//!   receive historical samples)
//!
//! - [`reliability`](qos::policy::Reliability) (best-effort vs reliable
//!   delivery)
//!
//! - [`history depth`](qos::policy::History) (the number of samples to store in
//!   history)
//!
//! - [`deadline`](qos::policy::Deadline) (whether a signal should be generated
//!   when a sample is not received within a specified period)
//!
//! Policies are set independently on the writer and reader side, and
//! compatibility is checked at discovery time. A writer's offered [`QoS`] must
//! be compatible with a reader's requested [`QoS`] for the two endpoints to
//! match.
//!
//! There are a variety of other elements to the DDS API such as:
//!
//! [`WaitSets`](WaitSet): to allow you to block until a particular status
//! occurs on a DDS entity. [`Listeners`](Listener): to notify applications of a
//! change in the status of a particular entity.
//! [`GuardConditions`](GuardCondition), `StatusConditions`,
//! [`ReadConditions`](ReadCondition), and [`QueryConditions`](QueryCondition):
//! Mechanisms to trigger the condition associated with a waitset.
//!
//! See the [DDS Specification](https://www.omg.org/spec/DDS/1.4/About-DDS/) and the [OMG DDS Wiki](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:01_front:4_toc) for these other elements and see the rest of the Rust Documentation for what is supported by this API.//!
//!
//! # Getting started
//!
//! Every DDS application begins with a [`Domain`] and a [`Participant`]:
//!
//! ```
//! use cyclonedds::{Domain, Participant};
//!
//! let domain = Domain::default();
//! let participant = Participant::new(&domain)?;
//! # Ok::<_, cyclonedds::Error>(())
//! ```
//!
//! Types that can be used as a topic payload must implement the [`Topicable`]
//! trait, either manually or via the
//! [`Topicable`](cyclonedds_macros::Topicable) derive macro. Once you have a
//! topic, create a [`Writer`] or [`Reader`] directly via `new` or through their
//! builders to set [`QoS`] or to associate specific publishers or subscribers.
//! You can then create samples and write them via the writer and read those
//! samples back via the reader.
//!
//! ```
//! # use cyclonedds::{Domain, Participant};
//! # let domain = Domain::default();
//! # let participant = Participant::new(&domain)?;
//! # #[derive(
//! #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
//! # )]
//! # struct MyData {
//! #     x: i32,
//! # }
//! use cyclonedds::qos;
//! use cyclonedds::{QoS, Reader, Subscriber, Topic, Writer};
//!
//! let topic = Topic::<MyData>::new(&participant, "MyTopic")?;
//!
//! let qos = QoS::new()
//!     .with_reliability(qos::policy::Reliability::BestEffort)
//!     .with_history(qos::policy::History::KeepLast { depth: 10 });
//!
//! let subscriber = Subscriber::builder(&participant).with_qos(&qos).build()?;
//!
//! let writer = Writer::builder(&topic).with_qos(&qos).build()?;
//! let reader = Reader::builder(&topic)
//!     .with_qos(&qos)
//!     .with_subscriber(&subscriber)
//!     .build()?;
//!
//! for x in 0..10 {
//!     let sample = MyData { x };
//!     writer.write(&sample)?;
//! }
//! # assert_eq!(10, reader.read()?.len());
//!
//! // Does not remove the samples from the history,
//! // and does not update metadata.
//! for sample in reader.peek()? {
//!     // process sample
//! }
//!
//! // Does not remove the samples from the history,
//! // but does update metadata.
//! for sample in reader.read()? {
//!     // process sample
//! }
//!
//! // Removes the samples from the history,
//! // and updates metadata.
//! for sample in reader.take()? {
//!     // process sample
//! }
//!
//! # assert_eq!(0, reader.read()?.len());
//! # Ok::<_, cyclonedds::Error>(())
//! ```
//!
//! For further reading, see the [Cyclone DDS
//! documentation](https://cyclonedds.io), the [OMG DDS
//! specification](https://www.omg.org/spec/DDS/), and the
//! [`examples`](https://github.com/eclipse-cyclonedds/cyclonedds-rust/tree/master/cyclonedds/examples).

// NOTE: this is specified here rather than in the common lints within the workspace `Cargo.toml`
// because excluding it for the examples and integration tests is problematic.
#![deny(unused_crate_dependencies)]
// NOTE: active lint levels are defined in the workspace `Cargo.toml`. J
// These `allow`s for the test exist for lints that significantly reduce test readability or
// ergonomics.
#![cfg_attr(
    test,
    allow(
        clippy::cast_sign_loss,
        clippy::cognitive_complexity,
        clippy::indexing_slicing,
        clippy::too_many_lines,
        clippy::undocumented_unsafe_blocks,
    )
)]

pub mod cdr_bounds;
mod domain;
mod duration;
pub mod entity;
mod error;
mod guard_condition;
pub mod listener;
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
pub use listener::{
    Listener, PublisherListener, ReaderListener, SubscriberListener, TopicListener, WriterListener,
};
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
