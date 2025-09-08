use crate::internal::ffi;
use crate::{Reader, Result, State};

/// A filter on a [`Reader`](crate::Reader) that restricts samples by their
/// [`State`](crate::State).
///
/// A `ReadCondition` is created against a reader with a state mask and can be
/// attached to a [`WaitSet`](crate::WaitSet) to trigger when matching samples
/// become available. Reading via the condition returns only samples whose
/// combined sample, view, and instance state matches the mask.
///
/// # Examples
///
/// ```no_run
/// use cyclonedds::state;
/// use cyclonedds::{Duration, ReadCondition, WaitSet};
/// # use cyclonedds::{Domain, Participant, Topic, Reader};
/// # let domain = Domain::default();
/// # let participant = Participant::new(&domain)?;
/// # #[derive(
/// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
/// # )]
/// # struct Data {
/// #     x: i32,
/// #     y: i32,
/// # }
///
/// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
/// let reader = Reader::new(&topic)?;
///
/// let condition = ReadCondition::new(
///     &reader,
///     state::sample::Fresh | state::instance::Any | state::view::Any,
/// )?;
/// let mut waitset = WaitSet::<()>::new(&participant)?;
/// waitset.attach(&condition, None)?;
/// waitset.wait(Duration::INFINITE)?;
///
/// let samples = condition.take()?;
/// # Ok::<_, cyclonedds::Error>(())
/// ```
#[derive(Debug)]
pub struct ReadCondition<'domain, 'participant, 'topic, 'reader, T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom: std::marker::PhantomData<&'reader Reader<'domain, 'participant, 'topic, T>>,
}

impl<'d, 'p, 't, 'r, T> ReadCondition<'d, 'p, 't, 'r, T>
where
    T: crate::Topicable,
{
    /// Creates a new [`ReadCondition`] on `reader` that matches samples whose
    /// state satisfies `mask`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the read condition fails to
    /// create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let condition = ReadCondition::new(&reader, state::sample::Fresh)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(reader: &'r Reader<'d, 'p, 't, T>, mask: State) -> Result<Self> {
        let inner = ffi::dds_create_readcondition(reader.inner, mask.bits())?;
        Ok(Self {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    /// Returns the state mask this condition was created with.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the mask returned by the read
    /// condition is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let condition = ReadCondition::new(&reader, state::sample::Fresh)?;
    /// assert_eq!(condition.mask()?, state::sample::Fresh);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn mask(&self) -> Result<State> {
        let mask = ffi::dds_get_mask(self.inner)?;
        crate::state::State::from_bits(mask).ok_or(crate::error::Error::NonSpecific)
    }

    /// Returns `true` if this condition is currently triggered.
    ///
    /// A condition is triggered when samples matching its mask are available
    /// in the reader cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the read condition fails to read
    /// the trigger state.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = ReadCondition::new(&reader, state::sample::Fresh)?;
    /// writer.write(&Data::default())?;
    /// assert!(condition.triggered()?);
    /// Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn triggered(&self) -> Result<bool> {
        ffi::dds_triggered(self.inner)
    }

    /// Removes and returns all samples matching this condition's mask from the
    /// reader cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the read condition fails to take
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = ReadCondition::new(
    ///     &reader,
    ///     state::sample::Stale | state::instance::Any | state::view::Any,
    /// )?;
    /// writer.write(&Data::default())?;
    ///
    /// // No sample matches this state initially.
    /// let samples = condition.take()?;
    /// assert_eq!(samples.len(), 0);
    ///
    /// // Attempt a normal read.
    /// assert_eq!(reader.read()?.len(), 1);
    ///
    /// // Sample should now match this state because they're stale.
    /// let samples = condition.take()?;
    /// assert_eq!(samples.len(), 1);
    ///
    /// // Samples should be removed from the cache.
    /// assert_eq!(condition.take()?.len(), 0);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn take(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_take(self.inner)
    }

    /// Returns all samples matching this condition's mask without removing
    /// them from the reader cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the read condition fails to read
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = ReadCondition::new(
    ///     &reader,
    ///     state::sample::Stale | state::instance::Any | state::view::Any,
    /// )?;
    /// writer.write(&Data::default())?;
    ///
    /// // No sample matches this state initially.
    /// let samples = condition.read()?;
    /// assert_eq!(samples.len(), 0);
    ///
    /// // Attempt a normal read.
    /// assert_eq!(reader.read()?.len(), 1);
    ///
    /// // Sample should now match this state because they're stale.
    /// let samples = condition.read()?;
    /// assert_eq!(samples.len(), 1);
    ///
    /// // Samples remain in the cache.
    /// assert_eq!(condition.read()?.len(), 1);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn read(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_read(self.inner)
    }

    /// Returns all samples matching this condition's mask without marking them
    /// as read or removing them from the cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the read condition fails to peek
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{ReadCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = ReadCondition::new(
    ///     &reader,
    ///     state::sample::Stale | state::instance::Any | state::view::Any,
    /// )?;
    /// writer.write(&Data::default())?;
    ///
    /// // No sample matches this state initially.
    /// let samples = condition.peek()?;
    /// assert_eq!(samples.len(), 0);
    ///
    /// // Attempt a normal read.
    /// assert_eq!(reader.read()?.len(), 1);
    ///
    /// // Sample should now match this state because they're stale.
    /// let samples = condition.peek()?;
    /// assert_eq!(samples.len(), 1);
    ///
    /// // Samples remain in the cache.
    /// assert_eq!(condition.peek()?.len(), 1);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn peek(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_peek(self.inner)
    }
}

impl<T> Drop for ReadCondition<'_, '_, '_, '_, T>
where
    T: crate::Topicable,
{
    fn drop(&mut self) {
        let result = ffi::dds_delete(self.inner);
        debug_assert!(
            result.is_ok(),
            "unable to delete {self:?}: failed with {result:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;

    #[test]
    fn test_read_condition_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let _ = ReadCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
        )
        .unwrap();
    }

    #[test]
    fn test_read_condition_create_with_invalid_reader() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let mut reader = crate::Reader::new(&topic).unwrap();
        let reader_id = reader.inner;
        reader.inner = 0;
        let result = ReadCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
        )
        .unwrap_err();
        reader.inner = reader_id;
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_read_condition_get_mask() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();

        let mask = state::sample::Any | state::instance::Any | state::view::Any;

        let read_condition = ReadCondition::new(&reader, mask).unwrap();
        let result = read_condition.mask().unwrap();
        assert_eq!(result, mask);

        let mask = state::sample::Fresh | state::instance::Unregistered | state::view::Old;
        let result = read_condition.mask().unwrap();
        assert_ne!(result, mask);

        let read_condition = ReadCondition::new(&reader, mask).unwrap();
        let result = read_condition.mask().unwrap();
        assert_eq!(result, mask);
    }

    #[test]
    fn test_read_condition_get_mask_on_invalid_read_condition() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let mut read_condition = ReadCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
        )
        .unwrap();
        let read_condition_id = read_condition.inner;
        read_condition.inner = 0;
        let result = read_condition.mask().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = read_condition.triggered().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        read_condition.inner = read_condition_id;
    }

    #[test]
    fn test_read_condition_triggering_reads() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();

        let mask = state::sample::Stale | state::instance::Any | state::view::Any;

        let read_condition = ReadCondition::new(&reader, mask).unwrap();

        let sample = crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: "hello".to_string(),
        };
        writer.write(&sample).unwrap();

        let read_condition_received = read_condition.read().unwrap();
        assert_eq!(read_condition_received.len(), 0);
        let triggered = read_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert_eq!(reader_received.len(), 1);
        assert_eq!(*reader_received[0], sample);
        assert_eq!(
            reader_received[0].info().state,
            state::sample::Fresh | state::view::New | state::instance::Alive
        );

        let triggered = read_condition.triggered().unwrap();
        assert!(triggered);

        let read_condition_received = read_condition.peek().unwrap();
        assert_eq!(read_condition_received.len(), 1);
        assert_eq!(*read_condition_received[0], sample);

        let triggered = read_condition.triggered().unwrap();
        assert!(triggered);

        let read_condition_received = read_condition.take().unwrap();
        assert_eq!(read_condition_received.len(), 1);
        assert_eq!(*read_condition_received[0], sample);

        let triggered = read_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert!(reader_received.is_empty());

        let read_condition_received = read_condition.read().unwrap();
        assert!(read_condition_received.is_empty());
    }
}
