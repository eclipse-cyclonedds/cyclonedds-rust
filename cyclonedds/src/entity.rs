//! The base of the DDS entity hierarchy.
//!
//! Most DDS objects ([`Participant`](crate::Participant),
//! [`Topic`](crate::Topic), [`Reader`](crate::Reader),
//! [`Writer`](crate::Writer), and others) are entities. See the
//! [implementors of `Entity`](Entity#implementors) for the full list. This
//! module provides the [`Entity`] trait with the common methods available to
//! all entities.

use crate::internal::ffi;
use crate::{Result, Status};

/// A unique opaque handle identifying an instance.
///
/// For keyed topics this corresponds to a specific key value, but applications
/// should treat it as an opaque DDS handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct InstanceHandle {
    pub(crate) inner: cyclonedds_sys::dds_instance_handle_t,
}

/// A raw entity ID for an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct EntityId {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
}

// TODO should the Entity trait be sealed?
/// Common interface implemented by all members of the DDS entity hierarchy.
///
/// - [`Participant`](crate::Participant): the root entity representing
///   membership in a domain.
///   - [`WaitSet`](crate::WaitSet): blocks until one or more attached
///     conditions are triggered.
///   - [`GuardCondition`](crate::GuardCondition): a manually triggered
///     condition for use with a [`WaitSet`](crate::WaitSet).
///   - [`Topic<T>`](crate::Topic): names and types a data channel for a
///     specific payload type `T`.
///   - [`Publisher`](crate::Publisher): groups [`Writers`](crate::Writer) and
///     controls their shared [`QoS`](crate::QoS).
///     - [`Writer<T>`](crate::Writer): publishes samples of type `T` to a
///       [`Topic`](crate::Topic).
///   - [`Subscriber`](crate::Subscriber): groups [`Readers`](crate::Reader) and
///     controls their shared [`QoS`](crate::QoS).
///     - [`Reader<T>`](crate::Reader): receives samples of type `T` from a
///       [`Topic`](crate::Topic).
///       - [`ReadCondition<T>`](crate::ReadCondition): filters
///         [`Reader`](crate::Reader) samples by
///         [`sample`](crate::state::sample), [`view`](crate::state::view), and
///         [`instance`](crate::state::instance) state.
///       - [`QueryCondition<T, F>`](crate::QueryCondition): filters
///         [`Reader`](crate::Reader) samples by [`sample state`](crate::State)
///         and a predicate.
pub trait Entity {
    /// Returns the [`EntityId`] of this entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Reader, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // The reader and the writer have distinct IDs.
    /// assert_ne!(reader.id(), writer.id());
    ///
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn id(&self) -> EntityId;

    /// Returns the [`InstanceHandle`] of this entity.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) specifying the reason if the instance
    /// handle fails to be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Reader, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // The reader and the writer have distinct instance handles.
    /// assert_ne!(reader.instance_handle()?, writer.instance_handle()?);
    ///
    /// // Instance handles can be used to identify entities across various API
    /// // calls. For example, the writer's handle appears in the set of matched
    /// // publications.
    /// let matched = reader.matched_publications()?;
    /// assert_eq!(matched[0], writer.instance_handle()?);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn instance_handle(&self) -> Result<InstanceHandle> {
        let entity = self.id();
        let inner = ffi::dds_get_instance_handle(entity.inner)?;
        Ok(InstanceHandle { inner })
    }

    /// Returns the set of status flags that have changed since they were last
    /// [`read`](crate::Reader::read) or [`taken`](crate::Reader::take).
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status bits of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists).
    ///
    /// - Returns [`BadParameter`](crate::Error::BadParameter) if the retrieved
    ///   bits do not correspond to a valid [`Status`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Reader, Status, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let reader = Reader::new(&topic)?;
    ///
    /// // The reader has been created but nothing in particular has happened in
    /// // terms of status changes.
    /// let changed = reader.status_changes()?;
    /// assert_eq!(changed, Status::empty());
    ///
    /// // The writer that is created will match with the reader.
    /// let writer = Writer::new(&topic)?;
    ///
    /// // After a writer matches, the reader reports a status change.
    /// let changed = reader.status_changes()?;
    /// assert!(changed.contains(Status::SubscriptionMatched));
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn status_changes(&self) -> Result<Status> {
        let entity = self.id();
        let status = ffi::dds_get_status_changes(entity.inner)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    /// Takes and clears the status flags matching `mask`, or all flags if
    /// `mask` is `None`.
    ///
    /// Unlike [`read_status`](Entity::read_status), this clears the returned
    /// flags on the entity.
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status bits of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists or the status mask contains entries that do not apply to the
    ///   entity type).
    ///
    /// - Returns [`BadParameter`](crate::Error::BadParameter) if the retrieved
    ///   bits do not correspond to a valid [`Status`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Reader, Status, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // The reader has matched with the writer, so its status should have
    /// // updated.
    /// let status = reader.take_status(Some(Status::SubscriptionMatched))?;
    /// assert!(status.contains(Status::SubscriptionMatched));
    ///
    /// // The flag has been cleared; a second take returns empty.
    /// let cleared = reader.take_status(Some(Status::SubscriptionMatched))?;
    /// assert!(cleared.is_empty());
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn take_status(&self, mask: Option<Status>) -> Result<Status> {
        let entity = self.id();
        let mask = mask.unwrap_or(Status::all()).bits();
        let status = ffi::dds_take_status(entity.inner, mask)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    /// Reads the status flags matching `mask` without clearing them, or all
    /// flags if `mask` is `None`.
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status bits of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists).
    ///
    /// - Returns [`BadParameter`](crate::Error::BadParameter) if the retrieved
    ///   bits do not correspond to a valid [`Status`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Reader, Status, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // The reader has matched with the writer, so its status should have
    /// // updated.
    /// let status = reader.read_status(Some(Status::SubscriptionMatched))?;
    /// assert!(status.contains(Status::SubscriptionMatched));
    ///
    /// // The flag is preserved; a second read returns the same value.
    /// let same = reader.read_status(Some(Status::SubscriptionMatched))?;
    /// assert_eq!(status, same);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn read_status(&self, mask: Option<Status>) -> Result<Status> {
        let entity = self.id();
        let mask = mask.unwrap_or(Status::all()).bits();
        let status = ffi::dds_read_status(entity.inner, mask)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    /// Returns the status mask enabled on the entity.
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status mask of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists).
    ///
    /// - Returns [`BadParameter`](crate::Error::BadParameter) if the retrieved
    ///   bits do not correspond to a valid [`Status`].
    ///
    /// # Examples
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Status, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // Get the initial active status mask.
    /// assert_eq!(
    ///     writer.status_mask()?,
    ///     Status::OfferedDeadlineMissed
    ///         | Status::OfferedIncompatibleQoS
    ///         | Status::LivelinessLost
    ///         | Status::PublicationMatched
    /// );
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn status_mask(&self) -> Result<Status> {
        let entity = self.id();
        let mask = ffi::dds_get_status_mask(entity.inner)?;
        Status::from_bits(mask).ok_or(crate::error::Error::BadParameter)
    }

    /// Sets and enables a status mask on the entity.
    ///
    /// Only status flags included in `mask` will trigger listener callbacks or
    /// be reported via [`status_changes`](Entity::status_changes).
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status mask of the
    ///   corresponding entity could not be set (e.g. the entity no longer
    ///   exists).
    ///
    /// # Examples
    /// ```
    /// use cyclonedds::entity::Entity;
    /// use cyclonedds::{Status, Topic, Writer};
    ///
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// # let domain = cyclonedds::Domain::default();
    /// # let participant = cyclonedds::Participant::new(&domain)?;
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// // Set the active status mask.
    /// writer.set_status_mask(Status::PublicationMatched)?;
    /// // Get the active status mask.
    /// assert_eq!(writer.status_mask()?, Status::PublicationMatched);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    fn set_status_mask(&self, mask: Status) -> Result<()> {
        let entity = self.id();
        let mask = mask.bits();
        ffi::dds_set_status_mask(entity.inner, mask)
    }
}

macro_rules! impl_entity {
    ($ty:ty) => {
        impl Entity for $ty {
            fn id(&self) -> EntityId {
                EntityId { inner: self.inner }
            }
        }
    };
    ($ty:ty where $($bounds:tt)*) => {
        impl<$($bounds)*> Entity for $ty {
            fn id(&self) -> EntityId {
                EntityId { inner: self.inner }
            }
        }
    };
}

impl_entity!(crate::Participant<'_>);
impl_entity!(crate::Topic<'_, '_, T> where T: crate::Topicable);
impl_entity!(crate::Publisher<'_, '_>);
impl_entity!(crate::Subscriber<'_, '_>);
impl_entity!(crate::Reader<'_, '_, '_, T> where T: crate::Topicable);
impl_entity!(crate::Writer<'_, '_, '_, T> where T: crate::Topicable);
impl_entity!(crate::ReadCondition<'_, '_, '_, '_, T> where T: crate::Topicable);
impl_entity!(crate::QueryCondition<'_, '_, '_, '_, T, F> where T: crate::Topicable, F: Fn(&T) -> bool);
impl_entity!(crate::GuardCondition<'_>);
impl_entity!(crate::WaitSet<'_, '_, '_, A> where A);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_all_entity_types() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let publisher = crate::Publisher::new(&participant).unwrap();
        let subscriber = crate::Subscriber::new(&participant).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();
        let read_condition = crate::ReadCondition::new(&reader, crate::state::sample::Any).unwrap();
        let query_condition =
            crate::QueryCondition::new(&reader, crate::State::empty(), |_| true).unwrap();
        let guard_condition = crate::GuardCondition::new(&participant).unwrap();
        let waitset = crate::WaitSet::<()>::new(&participant).unwrap();

        assert_eq!(participant.id().inner, participant.inner);
        assert_eq!(topic.id().inner, topic.inner);
        assert_eq!(publisher.id().inner, publisher.inner);
        assert_eq!(subscriber.id().inner, subscriber.inner);
        assert_eq!(reader.id().inner, reader.inner);
        assert_eq!(writer.id().inner, writer.inner);
        assert_eq!(read_condition.id().inner, read_condition.inner);
        assert_eq!(query_condition.id().inner, query_condition.inner);
        assert_eq!(guard_condition.id().inner, guard_condition.inner);
        assert_eq!(waitset.id().inner, waitset.inner);
    }

    #[test]
    fn test_entity_methods_on_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let mut participant = crate::Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;

        assert_eq!(
            crate::Error::BadParameter,
            participant.instance_handle().unwrap_err()
        );
        assert_eq!(
            crate::Error::BadParameter,
            participant.status_changes().unwrap_err()
        );
        assert_eq!(
            crate::Error::BadParameter,
            participant.take_status(None).unwrap_err()
        );
        assert_eq!(
            crate::Error::BadParameter,
            participant.read_status(None).unwrap_err()
        );
        assert_eq!(
            crate::Error::BadParameter,
            participant.status_mask().unwrap_err()
        );
        assert_eq!(
            crate::Error::BadParameter,
            participant
                .set_status_mask(crate::Status::InconsistentTopic)
                .unwrap_err()
        );

        participant.inner = participant_id;
    }

    #[test]
    fn test_entity_methods_on_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let result = participant.instance_handle();
        assert!(result.is_ok());
        let status_changes = participant.status_changes().unwrap();
        assert!(status_changes.is_empty());
        let result = participant.set_status_mask(crate::Status::empty());
        assert!(result.is_ok());
        let mask = participant.status_mask().unwrap();
        assert_eq!(mask, crate::Status::empty());
        let status = participant
            .read_status(Some(crate::Status::empty()))
            .unwrap();
        assert!(status.is_empty());
        let status = participant
            .take_status(Some(crate::Status::empty()))
            .unwrap();
        assert!(status.is_empty());
    }

    #[test]
    fn test_entity_methods_on_reader() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();

        let result = reader.instance_handle();
        assert!(result.is_ok());
        let status_changes = reader.status_changes().unwrap();
        assert!(status_changes.is_empty());
        let result = reader.set_status_mask(crate::Status::SubscriptionMatched);
        assert!(result.is_ok());
        let mask = reader.status_mask().unwrap();
        assert_eq!(mask, crate::Status::SubscriptionMatched);
        let status = reader
            .read_status(Some(crate::Status::SubscriptionMatched))
            .unwrap();
        assert!(status.is_empty());
        let status = reader
            .take_status(Some(crate::Status::SubscriptionMatched))
            .unwrap();
        assert!(status.is_empty());
    }
}
