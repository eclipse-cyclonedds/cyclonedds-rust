//! How a topic exchanges its samples.
//!
//! Every [`Topic`](crate::Topic) carries one of these markers as a type
//! parameter, which decides how its samples travel and what its endpoints can
//! do. [`Serialized`] is the default and needs no mention; [`ZeroCopy`] is
//! reached through
//! [`TopicBuilder::with_zero_copy`](crate::builder::TopicBuilder::with_zero_copy)
//! and carries through to the [`Writer`](crate::Writer) and
//! [`Reader`](crate::Reader) built from the topic.
//!
//! The distinction is a type parameter rather than a flag so that operations
//! which only make sense for one of them — [`Writer::loan`](crate::Writer::loan)
//! and [`Reader::take_loaned`](crate::Reader::take_loaned), which hand out
//! borrowed shared memory — simply do not exist on the other.

/// A topic's sample representation.
///
/// Implemented by [`Serialized`] and [`ZeroCopy`], and sealed: a representation
/// is a claim about how Cyclone DDS may handle a type's memory, so the set of
/// them is fixed by this crate.
pub trait Representation:
    sealed::Sealed + Copy + std::fmt::Debug + Default + Send + Sync + 'static
{
}

impl Representation for Serialized {}
impl Representation for ZeroCopy {}

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Serialized {}
    impl Sealed for super::ZeroCopy {}
}

/// Samples are exchanged as CDR.
///
/// The writer serializes each sample and the reader deserializes it, which is
/// what lets any two DDS implementations understand each other. This applies
/// however the bytes travel, including over shared memory: a
/// [`Serialized`] topic with a shared-memory PSMX Instance skips the network
/// stack but still encodes and decodes.
///
/// Available for every [`Topicable`](crate::Topicable) type, and the default for
/// every topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Serialized;

/// Samples are exchanged as raw memory.
///
/// Cyclone DDS is told the type is `memcpy`-safe, so a sample published into a
/// shared-memory PSMX loan reaches readers on the same machine without being
/// copied or converted at all. Peers that cannot take that route — a remote
/// participant, or a local one without a shared-memory PSMX Instance — still
/// exchange CDR, so a zero-copy topic remains interoperable.
///
/// Only available for a [`FixedSizeTopic`](crate::FixedSizeTopic), whose
/// contract is what makes another process's bytes safe to read as a `T`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ZeroCopy;
