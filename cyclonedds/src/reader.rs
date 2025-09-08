use crate::internal::ffi;
use crate::{Result, Subscriber, Topic};

/// A data reader for topic type [`T`](crate::Topicable).
///
/// A `Reader` receives samples of type [`T`](crate::Topicable) from a named
/// [`Topic`](crate::Topic). Samples are retrieved via [`read`](Reader::read),
/// [`take`](Reader::take), or [`peek`](Reader::peek). Matched
/// [`Writers`](crate::Writer) on the same topic deliver samples subject to
/// [`QoS`](crate::QoS) compatibility.
///
/// Use [`Reader::new`] for simple construction or [`Reader::builder`] for
/// [`QoS`](crate::QoS) and [`listener`](crate::listener::ReaderListener)
/// configuration.
///
/// # `peek` vs `read` vs `take`
///
/// | Method                 | Behavior                                                                                 | Cache effect                               | Read state effect              |
/// |------------------------|------------------------------------------------------------------------------------------|--------------------------------------------|--------------------------------|
/// | [`peek`](Reader::peek) | Returns samples without consuming them. Useful for checking whether data is available.   | Samples remain in the reader cache.        | Stays unread.                  |
/// | [`read`](Reader::read) | Returns samples and marks them as read (but leaves them available for subsequent reads). | Samples remain in the reader cache.        | Marked as read.                |
/// | [`take`](Reader::take) | Returns samples and removes them (making them unavailable for subsequent reads).         | Samples are removed from the reader cache. | Consumed and no longer cached. |
#[derive(Debug, PartialEq, Eq)]
pub struct Reader<'domain, 'participant, 'topic, T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom_topic: std::marker::PhantomData<&'topic Topic<'domain, 'participant, T>>,
}

/// Builder for [`Reader<T>`] (accessible via [`Reader::builder`]).
#[derive(Debug)]
pub struct ReaderBuilder<'domain, 'participant, 'topic, 'qos, T>
where
    T: crate::Topicable,
{
    subscriber: Option<&'participant Subscriber<'domain, 'participant>>,
    topic: &'topic Topic<'domain, 'participant, T>,
    qos: Option<&'qos crate::QoS>,
}

impl<'d, 'p, 't, 'q, T> ReaderBuilder<'d, 'p, 't, 'q, T>
where
    T: crate::Topicable,
{
    /// Creates a new [`ReaderBuilder`] for the given [`Topic`].
    #[must_use]
    pub const fn new(topic: &'t Topic<'d, 'p, T>) -> Self {
        Self {
            subscriber: None,
            topic,
            qos: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this reader builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Sets the [`Subscriber`](crate::Subscriber) on this reader builder.
    #[must_use]
    pub const fn with_subscriber(mut self, subscriber: &'p Subscriber<'d, 'p>) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    /// Builds the [`Reader`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader failed to create.
    pub fn build(self) -> Result<Reader<'d, 'p, 't, T>> {
        Ok(Reader {
            inner: ffi::dds_create_reader(
                self.subscriber
                    .map_or(ffi::dds_get_participant(self.topic.inner)?, |subscriber| {
                        subscriber.inner
                    }),
                self.topic.inner,
                self.qos.map(|qos| &qos.inner),
                None,
            )?,
            phantom_topic: std::marker::PhantomData,
        })
    }
}

impl<'d, 'p, 't, T> Reader<'d, 'p, 't, T>
where
    T: crate::Topicable,
{
    /// Creates a new `Reader` for the given [`Topic`](crate::Topic) with
    /// default [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::ReaderListener).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader fails to create.
    pub fn new(topic: &'t Topic<'d, 'p, T>) -> Result<Self> {
        Self::builder(topic).build()
    }

    /// Returns a [`ReaderBuilder`](crate::builder::ReaderBuilder) for
    /// constructing a reader with custom [`QoS`](crate::QoS) or a
    /// [`listener`](crate::listener::ReaderListener).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QoS, Reader, qos::policy::History};
    ///
    /// # use cyclonedds::{Domain, Participant, Topic};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "Example")?;
    /// let qos = QoS::new().with_history(History::KeepAll);
    /// let reader = Reader::builder(&topic).with_qos(&qos).build()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn builder<'q>(topic: &'t Topic<'d, 'p, T>) -> ReaderBuilder<'d, 'p, 't, 'q, T> {
        ReaderBuilder::new(topic)
    }

    /// Removes and returns all available samples from the reader cache.
    ///
    /// Each call to `take` consumes the returned samples so they will not be
    /// returned by subsequent calls. See [`read`](Reader::read) to leave
    /// samples in the cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader fails to take samples.
    pub fn take(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_take(self.inner)
    }

    /// Returns all available samples from the reader cache without removing
    /// them.
    ///
    /// Samples returned by `read` remain in the cache and will be returned
    /// again by subsequent calls, marked as read in their
    /// [`Info`](crate::sample::Info) state. See [`take`](Reader::take) to
    /// consume samples.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader fails to read samples.
    pub fn read(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_read(self.inner)
    }

    /// Returns all available samples without marking them as read or removing
    /// them from the cache.
    ///
    /// Useful for checking whether data is available without affecting the
    /// read state of samples. Subsequent calls to [`read`](Reader::read) or
    /// [`take`](Reader::take) will still return the same samples as unread.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader fails to peek.
    pub fn peek(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_peek(self.inner)
    }

    /// Returns the instance handles of all writers currently matched with
    /// this reader.
    ///
    /// The returned handles can be compared against
    /// [`InstanceHandle`](crate::entity::InstanceHandle) values from writer
    /// entities to identify specific matched writers.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the reader fails to retrieve the
    /// matched publications.
    pub fn matched_publications(&self) -> Result<Vec<crate::entity::InstanceHandle>> {
        let matched = ffi::dds_get_matched_publications(self.inner)?;
        let matched = matched
            .iter()
            .map(|&inner| crate::entity::InstanceHandle { inner })
            .collect();
        Ok(matched)
    }

    /// Blocks until all historical data available from matched writers with
    /// [`TransientLocal`](crate::qos::policy::Durability::TransientLocal) or
    /// higher durability has been received, or until `timeout` elapses.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the timeout elapses before
    /// historical data is received or if the reader returns an error.
    pub fn wait_for_historical_data(&self, timeout: crate::Duration) -> Result<()> {
        ffi::dds_reader_wait_for_historical_data(self.inner, timeout.inner)
    }

    pub(crate) const fn from_existing(
        inner: cyclonedds_sys::dds_entity_t,
    ) -> std::mem::ManuallyDrop<Self> {
        std::mem::ManuallyDrop::new(Self {
            inner,
            phantom_topic: std::marker::PhantomData,
        })
    }
}

impl<T> Drop for Reader<'_, '_, '_, T>
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
    use crate::entity::Entity;

    #[test]
    fn test_reader_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let subscriber = crate::Subscriber::new(&participant).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let _ = Reader::new(&topic).unwrap();
        let _ = Reader::builder(&topic).with_qos(&qos).build().unwrap();
        let _ = Reader::builder(&topic)
            .with_subscriber(&subscriber)
            .build()
            .unwrap();
        let _ = Reader::builder(&topic)
            .with_qos(&qos)
            .with_subscriber(&subscriber)
            .build()
            .unwrap();
    }

    #[test]
    fn test_reader_create_with_invalid_topic() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let topic_id = topic.inner;
        topic.inner = 0;
        let result = Reader::new(&topic).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Reader::builder(&topic).with_qos(&qos).build().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        topic.inner = topic_id;
    }

    #[test]
    fn test_reader_create_with_invalid_subscriber() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut subscriber = crate::Subscriber::new(&participant).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let subscriber_id = subscriber.inner;
        subscriber.inner = 0;
        let result = Reader::builder(&topic)
            .with_subscriber(&subscriber)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        subscriber.inner = subscriber_id;
    }

    #[test]
    fn test_reader_empty_read() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader = Reader::new(&topic).unwrap();
        let _ = reader.read().unwrap();
    }

    #[test]
    fn test_reader_empty_take() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader = Reader::new(&topic).unwrap();
        let _ = reader.take().unwrap();
    }

    #[test]
    fn test_reader_empty_peek() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader = Reader::new(&topic).unwrap();
        let _ = reader.peek().unwrap();
    }

    #[test]
    fn test_reader_create_from_existing() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader_01 = Reader::new(&topic).unwrap();
        let reader_02 = Reader::<crate::tests::topic::Data>::from_existing(reader_01.inner);
        assert_eq!(reader_01.inner, reader_02.inner);
    }

    #[test]
    fn test_reader_wait_for_historical_data() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader = Reader::new(&topic).unwrap();

        reader
            .wait_for_historical_data(crate::Duration::INFINITE)
            .unwrap();
    }

    #[test]
    fn test_reader_matched_publications() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let reader = Reader::new(&topic).unwrap();
        let matched = reader.matched_publications().unwrap();
        assert_eq!(matched.len(), 0);

        let writer = crate::Writer::new(&topic).unwrap();

        let matched = reader.matched_publications().unwrap();

        assert_eq!(matched.len(), 1);
        let expected = writer.instance_handle().unwrap();
        let actual = matched[0];
        assert_eq!(expected, actual);
    }
}
