use crate::internal::ffi;
use crate::{Publisher, Result, Topic};

/// A data writer for topic type [`T`](crate::Topicable).
///
/// A `Writer` publishes samples of type `T` to a named [`Topic`](crate::Topic).
/// Matched [`Readers`](crate::Reader) on the same topic receive the samples
/// subject to their [`QoS`](crate::QoS) compatibility.
///
/// Use [`Writer::new`] for simple construction or [`Writer::builder`] for
/// [`QoS`](crate::QoS), [`listener`](crate::listener::WriterListener), and
/// [`publisher`](Publisher) configuration.
///
/// # Instance lifecycle
///
/// For keyed topics, each unique key value identifies a distinct instance.
/// Writers can explicitly manage instance lifecycle through
/// [`register_instance`](Writer::register_instance),
/// [`unregister_instance`](Writer::unregister_instance), and
/// [`dispose`](Writer::dispose). Unkeyed topics (where
/// [`T::Key`](crate::Topicable::Key) is [`()`](primitive@unit)) have
/// a single instance shared by all samples.
#[derive(Debug, PartialEq, Eq)]
pub struct Writer<'domain, 'participant, 'topic, T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom_topic: std::marker::PhantomData<&'topic Topic<'domain, 'participant, T>>,
}

/// Builder for [`Writer<T>`] (accessible via [`Writer::builder`]).
#[derive(Debug)]
pub struct WriterBuilder<'domain, 'participant, 'topic, 'qos, T>
where
    T: crate::Topicable,
{
    publisher: Option<&'participant Publisher<'domain, 'participant>>,
    topic: &'topic Topic<'domain, 'participant, T>,
    qos: Option<&'qos crate::QoS>,
}

impl<'d, 'p, 't, 'q, T> WriterBuilder<'d, 'p, 't, 'q, T>
where
    T: crate::Topicable,
{
    /// Creates a new [`WriterBuilder`] for the given [`Topic`].
    #[must_use]
    pub const fn new(topic: &'t Topic<'d, 'p, T>) -> Self {
        Self {
            publisher: None,
            topic,
            qos: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this writer builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Sets the [`Publisher`](crate::Publisher) on this writer builder.
    #[must_use]
    pub const fn with_publisher(mut self, publisher: &'p Publisher<'d, 'p>) -> Self {
        self.publisher = Some(publisher);
        self
    }

    /// Builds the [`Writer`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer failed to create.
    pub fn build(self) -> Result<Writer<'d, 'p, 't, T>> {
        Ok(Writer {
            inner: ffi::dds_create_writer(
                self.publisher
                    .map_or(ffi::dds_get_participant(self.topic.inner)?, |publisher| {
                        publisher.inner
                    }),
                self.topic.inner,
                self.qos.map(|qos| &qos.inner),
                None,
            )?,
            phantom_topic: std::marker::PhantomData,
        })
    }
}

impl<'d, 'p, 't, T> Writer<'d, 'p, 't, T>
where
    T: crate::Topicable,
{
    /// Creates a new `Writer` for the given [`Topic`](crate::Topic) with
    /// default [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::WriterListener).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to create.
    pub fn new(topic: &'t Topic<'d, 'p, T>) -> Result<Self> {
        Self::builder(topic).build()
    }

    /// Returns a [`WriterBuilder`](crate::builder::WriterBuilder) for
    /// constructing a writer with custom [`QoS`](crate::QoS) or a
    /// [`listener`](crate::listener::WriterListener).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Duration, QoS, Writer, qos::policy::Reliability};
    /// # use cyclonedds::{Domain, Participant, Topic};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     #[dds(key)]
    /// #     x: i32,
    /// #     #[dds(key)]
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let qos = QoS::new().with_reliability(Reliability::Reliable {
    ///     max_blocking_time: Duration::from_millis(100),
    /// });
    /// let writer = Writer::builder(&topic).with_qos(&qos).build()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn builder<'q>(topic: &'t Topic<'d, 'p, T>) -> WriterBuilder<'d, 'p, 't, 'q, T> {
        WriterBuilder::new(topic)
    }

    /// Writes a sample to the topic.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writers fails to write the
    /// sample.
    pub fn write(&self, sample: &T) -> Result<()> {
        ffi::dds_write(self.inner, sample)
    }

    /// Writes a sample with an explicit source timestamp.
    ///
    /// Use this when the write timestamp should reflect the time the data was
    /// generated rather than the time it was written.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to write the
    /// sample.
    pub fn write_with_timestamp(&self, sample: &T, timestamp: crate::Time) -> Result<()> {
        ffi::dds_write_with_timestamp(self.inner, sample, timestamp.inner)
    }

    /// Flushes batched samples to the network.
    ///
    /// Only relevant when write batching is enabled in the domain
    /// configuration. Has no effect otherwise.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to flush.
    pub fn flush(&self) -> Result<()> {
        ffi::dds_write_flush(self.inner)
    }

    /// Blocks until all written samples have been acknowledged by all matched
    /// reliable readers, or until `timeout` elapses.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the timeout elapses before all
    /// acknowledgements are received or if the writer encounters an unexpected
    /// error.
    pub fn wait_for_acks(&self, timeout: crate::Duration) -> Result<()> {
        ffi::dds_wait_for_acks(self.inner, timeout.inner)
    }

    /// Returns the instance handles of all readers currently matched with
    /// this writer.
    ///
    /// The returned handles can be compared against
    /// [`InstanceHandle`](crate::entity::InstanceHandle) values from reader
    /// entities to identify specific matched readers.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to retrieve the
    /// matched subscriptions.
    pub fn matched_subscriptions(&self) -> Result<Vec<crate::entity::InstanceHandle>> {
        ffi::dds_get_matched_subscriptions(self.inner).map(|matched| {
            matched
                .iter()
                .map(|&inner| crate::entity::InstanceHandle { inner })
                .collect()
        })
    }

    /// Registers an instance identified by `key` with this writer.
    ///
    /// Registration is optional but allows for the pre-allocation of resources
    /// for the instance. Returns the
    /// [`InstanceHandle`](crate::entity::InstanceHandle) assigned to the
    /// instance.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to register the
    /// instance.
    pub fn register_instance(&self, key: &T::Key) -> Result<crate::entity::InstanceHandle> {
        ffi::dds_register_instance::<T>(self.inner, key)
            .map(|inner| crate::entity::InstanceHandle { inner })
    }

    /// Unregisters an instance identified by `key` from this writer.
    ///
    /// Notifies matched readers that this writer will no longer publish
    /// samples for the given instance.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to unregister
    /// the instance.
    pub fn unregister_instance(&self, key: &T::Key) -> Result<()> {
        ffi::dds_unregister_instance::<T>(self.inner, key)
    }

    /// Unregisters an instance identified by its
    /// [`InstanceHandle`](crate::entity::InstanceHandle).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to unregister
    /// the instance.
    pub fn unregister_instance_by_handle(
        &self,
        instance_handle: crate::entity::InstanceHandle,
    ) -> Result<()> {
        ffi::dds_unregister_instance_by_handle(self.inner, instance_handle.inner)
    }

    /// Unregisters an instance identified by `key` with an explicit timestamp.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to unregister
    /// the instance.
    pub fn unregister_instance_with_timestamp(
        &self,
        key: &T::Key,
        timestamp: crate::Time,
    ) -> Result<()> {
        ffi::dds_unregister_instance_with_timestamp::<T>(self.inner, key, timestamp.inner)
    }

    /// Unregisters an instance identified by its
    /// [`InstanceHandle`](crate::entity::InstanceHandle) with an explicit
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to unregister
    /// the instance.
    pub fn unregister_instance_by_handle_with_timestamp(
        &self,
        instance_handle: crate::entity::InstanceHandle,
        timestamp: crate::Time,
    ) -> Result<()> {
        ffi::dds_unregister_instance_by_handle_with_timestamp(
            self.inner,
            instance_handle.inner,
            timestamp.inner,
        )
    }

    /// Returns the [`InstanceHandle`](crate::entity::InstanceHandle) for the
    /// instance identified by `key`, or `None` if the instance is not
    /// registered.
    pub fn lookup_instance(&self, key: &T::Key) -> Option<crate::entity::InstanceHandle> {
        ffi::dds_lookup_instance::<T>(self.inner, key)
            .map(|inner| crate::entity::InstanceHandle { inner })
    }

    /// Writes a sample and immediately disposes the instance.
    ///
    /// Equivalent to calling [`write`](Writer::write) followed by
    /// [`dispose`](Writer::dispose) but in a single operation.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to write or
    /// dispose.
    pub fn write_dispose(&self, data: &T) -> Result<()> {
        ffi::dds_write_dispose(self.inner, data)
    }

    /// Writes a sample and immediately disposes the instance with an explicit
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to write or
    /// dispose.
    pub fn write_dispose_with_timestamp(&self, data: &T, timestamp: crate::Time) -> Result<()> {
        ffi::dds_write_dispose_with_timestamp(self.inner, data, timestamp.inner)
    }

    /// Disposes the instance identified by `key`.
    ///
    /// Notifies matched readers that the instance is no longer valid. The
    /// instance remains known but its state transitions to disposed.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to dispose the
    /// instance.
    pub fn dispose(&self, key: &T::Key) -> Result<()> {
        ffi::dds_dispose::<T>(self.inner, key)
    }

    /// Disposes the instance identified by `key` with an explicit timestamp.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to dispose the
    /// instance.
    pub fn dispose_with_timestamp(&self, key: &T::Key, timestamp: crate::Time) -> Result<()> {
        ffi::dds_dispose_with_timestamp::<T>(self.inner, key, timestamp.inner)
    }

    /// Disposes the instance identified by its
    /// [`InstanceHandle`](crate::entity::InstanceHandle).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to dispose the
    /// instance.
    pub fn dispose_instance_by_handle(
        &self,
        instance_handle: crate::entity::InstanceHandle,
    ) -> Result<()> {
        ffi::dds_dispose_instance_by_handle(self.inner, instance_handle.inner)
    }

    /// Disposes the instance identified by its
    /// [`InstanceHandle`](crate::entity::InstanceHandle) with an explicit
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the writer fails to dispose the
    /// instance.
    pub fn dispose_instance_by_handle_with_timestamp(
        &self,
        instance_handle: crate::entity::InstanceHandle,
        timestamp: crate::Time,
    ) -> Result<()> {
        ffi::dds_dispose_instance_by_handle_with_timestamp(
            self.inner,
            instance_handle.inner,
            timestamp.inner,
        )
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

impl<T> Drop for Writer<'_, '_, '_, T>
where
    T: crate::Topicable,
{
    fn drop(&mut self) {
        let result = ffi::dds_delete(self.inner);
        debug_assert!(
            result.is_ok(),
            "unable to delete {self:?}, failed with: {result:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Topicable;

    #[test]
    fn test_writer_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let publisher = crate::Publisher::new(&participant).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let _ = Writer::new(&topic).unwrap();
        let _ = Writer::builder(&topic).with_qos(&qos).build().unwrap();
        let _ = Writer::builder(&topic)
            .with_publisher(&publisher)
            .build()
            .unwrap();
        let _ = Writer::builder(&topic)
            .with_publisher(&publisher)
            .with_qos(&qos)
            .build()
            .unwrap();
    }

    #[test]
    fn test_writer_create_with_invalid_topic() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let topic_id = topic.inner;
        topic.inner = 0;
        let result = Writer::new(&topic).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Writer::builder(&topic).with_qos(&qos).build().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        topic.inner = topic_id;
    }

    #[test]
    fn test_writer_create_with_invalid_publisher() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut publisher = crate::Publisher::new(&participant).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let publisher_id = publisher.inner;
        publisher.inner = 0;
        let result = Writer::builder(&topic)
            .with_publisher(&publisher)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        publisher.inner = publisher_id;
    }

    #[test]
    fn test_writer_write() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let writer = Writer::new(&topic).unwrap();
        writer.write(&crate::tests::topic::Data::default()).unwrap();
    }

    #[test]
    fn test_writer_write_with_timestamp() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let writer = Writer::new(&topic).unwrap();
        let timestamp = crate::Time::from_nanos(10001);
        writer
            .write_with_timestamp(&crate::tests::topic::Data::default(), timestamp)
            .unwrap();
    }

    #[test]
    fn test_writer_operations_on_invalid_writer() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let mut writer = Writer::new(&topic).unwrap();

        let sample = crate::tests::topic::Data::default();
        let key = sample.as_key();
        let timestamp = crate::Time::from_nanos(10001);
        let instance_handle = crate::entity::InstanceHandle { inner: 0 };

        let writer_id = writer.inner;
        writer.inner = 0;

        let result = writer.write(&sample).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.write_dispose(&sample).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.write_with_timestamp(&sample, timestamp).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .write_dispose_with_timestamp(&sample, timestamp)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.unregister_instance(&key).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .unregister_instance_with_timestamp(&key, timestamp)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .unregister_instance_by_handle(instance_handle)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .unregister_instance_by_handle_with_timestamp(instance_handle, timestamp)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.dispose(&key).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.dispose_with_timestamp(&key, timestamp).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .dispose_instance_by_handle(instance_handle)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer
            .dispose_instance_by_handle_with_timestamp(instance_handle, timestamp)
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.flush().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.register_instance(&key).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = writer.matched_subscriptions().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        assert_eq!(result, crate::Error::BadParameter);
        writer.inner = writer_id;
    }

    #[test]
    fn test_writer_create_from_existing() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer_01 = Writer::new(&topic).unwrap();
        let writer_02 = Writer::<crate::tests::topic::Data>::from_existing(writer_01.inner);
        assert_eq!(writer_01.inner, writer_02.inner);
    }

    #[test]
    fn test_writer_register_unregister_instance() {
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer = Writer::new(&topic).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();

        let sample = crate::tests::topic::Data {
            x: 0,
            y: 1,
            message: String::from("initial"),
        };
        writer.write(&sample).unwrap();
        let sample = crate::tests::topic::Data {
            x: 1,
            y: 2,
            message: String::from("registered"),
        };
        writer.write(&sample).unwrap();
        let registered_handle = writer.register_instance(&sample.as_key()).unwrap();
        let sample = crate::tests::topic::Data {
            x: 2,
            y: 3,
            message: String::from("unregistered"),
        };
        writer.write(&sample).unwrap();
        writer.unregister_instance(&sample.as_key()).unwrap();

        writer.write(&crate::tests::topic::Data::default()).unwrap();

        let samples = reader.take().unwrap();
        assert_eq!(samples.len(), 4);

        for sample in samples {
            assert!(sample.is_sample());
            match sample.message.as_ref() {
                "initial" => {
                    assert_eq!(
                        (sample.x, sample.y, sample.message.as_ref()),
                        (0, 1, "initial")
                    );

                    assert_eq!(
                        sample.info().state,
                        state::sample::Fresh | state::view::New | state::instance::Alive
                    );
                }
                "registered" => {
                    assert_eq!(
                        (sample.x, sample.y, sample.message.as_ref()),
                        (1, 2, "registered")
                    );
                    let info = sample.info();
                    assert_eq!(info.instance_handle, registered_handle);
                    assert_eq!(
                        info.state,
                        state::sample::Fresh | state::view::New | state::instance::Alive
                    );
                }
                "unregistered" => {
                    assert_eq!(
                        (sample.x, sample.y, sample.message.as_ref()),
                        (2, 3, "unregistered")
                    );
                    let info = sample.info();
                    assert_eq!(
                        info.state,
                        state::sample::Fresh | state::view::New | state::instance::Disposed
                    );
                }
                _ => {
                    assert_eq!(*sample, crate::tests::topic::Data::default());
                    assert_eq!(
                        sample.info().state,
                        state::sample::Fresh | state::view::New | state::instance::Alive
                    );
                }
            }
        }

        let sample = crate::tests::topic::Data {
            x: 4,
            y: 5,
            message: String::from("registered"),
        };
        let key = sample.as_key();
        let registered_handle = writer.register_instance(&key).unwrap();
        writer.write(&sample).unwrap();
        let lookup_handle = writer.lookup_instance(&key).unwrap();
        assert_eq!(registered_handle, lookup_handle);
        writer
            .unregister_instance_by_handle(registered_handle)
            .unwrap();
        let received_sample = &reader.read().unwrap()[0];
        assert_eq!(**received_sample, sample);
        assert_eq!(
            received_sample.info().state,
            state::sample::Fresh | state::view::New | state::instance::Disposed
        );
    }

    // TODO this test doesn't really validate the flushing side.
    #[test]
    fn test_writer_flush() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer_01 = Writer::new(&topic).unwrap();
        let writer_02 = Writer::new(&topic).unwrap();

        let sample = crate::tests::topic::Data::default();

        writer_01.write(&sample).unwrap();
        writer_01.write(&sample).unwrap();
        writer_01.write(&sample).unwrap();
        writer_01.write(&sample).unwrap();
        writer_01.flush().unwrap();

        writer_02.write(&sample).unwrap();
        writer_02.write(&sample).unwrap();
        writer_02.write(&sample).unwrap();
        writer_02.write(&sample).unwrap();
        writer_02.flush().unwrap();
    }

    #[test]
    fn test_writer_wait_for_acks() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer = Writer::new(&topic).unwrap();
        let _reader = crate::Reader::builder(&topic)
            .with_qos(&crate::QoS::new().with_reliability(
                crate::qos::policy::Reliability::Reliable {
                    max_blocking_time: crate::Duration::INFINITE,
                },
            ))
            .build()
            .unwrap();

        writer.write(&crate::tests::topic::Data::default()).unwrap();
        writer
            .wait_for_acks(crate::Duration::from_nanos(100))
            .unwrap();
    }

    #[test]
    fn test_writer_matched_subscriptions() {
        use crate::entity::Entity;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer = Writer::new(&topic).unwrap();
        let matched = writer.matched_subscriptions().unwrap();
        assert!(matched.is_empty(), "{matched:#?}");
        let reader = crate::Reader::new(&topic).unwrap();
        let matched = writer.matched_subscriptions().unwrap();
        assert_eq!(matched, vec![reader.instance_handle().unwrap()]);
    }

    #[test]
    fn test_writer_lookup_instance() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        let writer = Writer::new(&topic).unwrap();

        let sample = crate::tests::topic::Data::default();
        let key = sample.as_key();

        let result = writer.lookup_instance(&key);
        assert_eq!(result, None);

        let registered_handle = writer.register_instance(&key).unwrap();
        let result = writer.lookup_instance(&key);
        assert_eq!(result, Some(registered_handle));
    }
    #[test]
    fn test_writer_unregister() {
        use crate::entity::Entity;
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos
            .with_reliability(crate::qos::policy::Reliability::Reliable {
                max_blocking_time: std::time::Duration::from_millis(100).try_into().unwrap(),
            })
            .with_resource_limits(crate::qos::policy::ResourceLimits {
                max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                max_instances: crate::qos::policy::ResourceLimit::Limited(3),
                max_samples_per_instance: crate::qos::policy::ResourceLimit::Limited(1),
            });

        let reader = crate::Reader::builder(&topic)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos.with_writer_data_lifecycle(crate::qos::policy::WriterDataLifecycle {
            autodispose_unregistered_instances: false,
        });
        let writer = Writer::builder(&topic).with_qos(&qos).build().unwrap();

        // Sync writer to reader.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        writer
            .set_status_mask(crate::Status::PublicationMatched)
            .unwrap();
        waitset.attach(&writer, Some(&writer)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &writer);
        waitset.detach(&writer).unwrap();

        // Sync reader to writer.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        reader
            .set_status_mask(crate::Status::SubscriptionMatched)
            .unwrap();
        waitset.attach(&reader, Some(&reader)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &reader);
        waitset.detach(&reader).unwrap();

        for i in 0..3 {
            let sample = crate::tests::topic::Data {
                x: i,
                y: i.cast_signed() + 1,
                ..crate::tests::topic::Data::default()
            };
            writer.write(&sample).unwrap();
        }

        let key_01 = crate::tests::topic::Data {
            x: 0,
            y: 1,
            ..crate::tests::topic::Data::default()
        }
        .as_key();
        let handle = writer.lookup_instance(&key_01).unwrap();
        writer.unregister_instance_by_handle(handle).unwrap();

        let key_02 = crate::tests::topic::Data {
            x: 1,
            y: 2,
            ..crate::tests::topic::Data::default()
        }
        .as_key();
        writer.unregister_instance(&key_02).unwrap();
        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 3);

        for sample in samples {
            let key = sample.as_key();

            if key == key_01 || key == key_02 {
                assert_eq!(*sample, crate::tests::topic::Data::from_key(&key));
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Unregistered
                );
            } else {
                assert_eq!(
                    *sample,
                    crate::tests::topic::Data {
                        x: 2,
                        y: 3,
                        ..crate::tests::topic::Data::default()
                    }
                );
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Alive
                );
            }
        }
    }

    #[test]
    fn test_writer_unregister_with_timestamp() {
        use crate::entity::Entity;
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos
            .with_reliability(crate::qos::policy::Reliability::Reliable {
                max_blocking_time: std::time::Duration::from_millis(100).try_into().unwrap(),
            })
            .with_resource_limits(crate::qos::policy::ResourceLimits {
                max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                max_instances: crate::qos::policy::ResourceLimit::Limited(3),
                max_samples_per_instance: crate::qos::policy::ResourceLimit::Limited(1),
            });

        let reader = crate::Reader::builder(&topic)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos.with_writer_data_lifecycle(crate::qos::policy::WriterDataLifecycle {
            autodispose_unregistered_instances: false,
        });
        let writer = Writer::builder(&topic).with_qos(&qos).build().unwrap();

        // Sync writer to reader.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        writer
            .set_status_mask(crate::Status::PublicationMatched)
            .unwrap();
        waitset.attach(&writer, Some(&writer)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &writer);
        waitset.detach(&writer).unwrap();

        // Sync reader to writer.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        reader
            .set_status_mask(crate::Status::SubscriptionMatched)
            .unwrap();
        waitset.attach(&reader, Some(&reader)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &reader);
        waitset.detach(&reader).unwrap();

        for i in 0..3 {
            let sample = crate::tests::topic::Data {
                x: i,
                y: i.cast_signed() + 1,
                ..Default::default()
            };
            writer.write(&sample).unwrap();
        }

        let time = std::time::SystemTime::now().try_into().unwrap();

        let key_01 = crate::tests::topic::Data {
            x: 0,
            y: 1,
            ..Default::default()
        }
        .as_key();
        let handle = writer.lookup_instance(&key_01).unwrap();
        writer
            .unregister_instance_by_handle_with_timestamp(handle, time)
            .unwrap();

        let key_02 = crate::tests::topic::Data {
            x: 1,
            y: 2,
            ..Default::default()
        }
        .as_key();
        writer
            .unregister_instance_with_timestamp(&key_02, time)
            .unwrap();
        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 3);

        for sample in samples {
            let key = sample.as_key();

            if key == key_01 || key == key_02 {
                assert_eq!(*sample, crate::tests::topic::Data::from_key(&key));
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Unregistered
                );
            } else {
                assert_eq!(
                    *sample,
                    crate::tests::topic::Data {
                        x: 2,
                        y: 3,
                        ..Default::default()
                    }
                );
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Alive
                );
            }
        }
    }

    #[test]
    fn test_writer_write_dispose() {
        use crate::entity::Entity;
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos
            .with_reliability(crate::qos::policy::Reliability::Reliable {
                max_blocking_time: std::time::Duration::from_millis(100).try_into().unwrap(),
            })
            .with_resource_limits(crate::qos::policy::ResourceLimits {
                max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                max_instances: crate::qos::policy::ResourceLimit::Limited(4),
                max_samples_per_instance: crate::qos::policy::ResourceLimit::Limited(1),
            });

        let reader = crate::Reader::builder(&topic)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos.with_writer_data_lifecycle(crate::qos::policy::WriterDataLifecycle {
            autodispose_unregistered_instances: false,
        });
        let writer = Writer::builder(&topic).with_qos(&qos).build().unwrap();

        // Sync writer to reader.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        writer
            .set_status_mask(crate::Status::PublicationMatched)
            .unwrap();
        waitset.attach(&writer, Some(&writer)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &writer);
        waitset.detach(&writer).unwrap();

        // Sync reader to writer.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        reader
            .set_status_mask(crate::Status::SubscriptionMatched)
            .unwrap();
        waitset.attach(&reader, Some(&reader)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &reader);
        waitset.detach(&reader).unwrap();

        let time = std::time::SystemTime::now().try_into().unwrap();
        for i in 0..4 {
            let sample = crate::tests::topic::Data {
                x: i,
                y: i.cast_signed() + 1,
                ..Default::default()
            };
            if sample.x.is_multiple_of(2) {
                if sample.x < 2 {
                    writer.write(&sample).unwrap();
                } else {
                    writer.write_with_timestamp(&sample, time).unwrap();
                }
            } else if sample.x < 2 {
                writer.write_dispose(&sample).unwrap();
            } else {
                writer.write_dispose_with_timestamp(&sample, time).unwrap();
            }
        }

        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 4);

        for sample in samples {
            assert_eq!(
                *sample,
                crate::tests::topic::Data {
                    x: sample.x,
                    y: sample.x.cast_signed() + 1,
                    ..Default::default()
                }
            );
            if sample.x % 2 == 0 {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Alive
                );
            } else {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Disposed
                );
            }
        }
    }

    #[test]
    fn test_writer_write_and_then_dispose() {
        use crate::entity::Entity;
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos
            .with_reliability(crate::qos::policy::Reliability::Reliable {
                max_blocking_time: std::time::Duration::from_millis(100).try_into().unwrap(),
            })
            .with_resource_limits(crate::qos::policy::ResourceLimits {
                max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                max_instances: crate::qos::policy::ResourceLimit::Limited(4),
                max_samples_per_instance: crate::qos::policy::ResourceLimit::Limited(1),
            });

        let reader = crate::Reader::builder(&topic)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos.with_writer_data_lifecycle(crate::qos::policy::WriterDataLifecycle {
            autodispose_unregistered_instances: false,
        });
        let writer = Writer::builder(&topic).with_qos(&qos).build().unwrap();

        // Sync writer to reader.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        writer
            .set_status_mask(crate::Status::PublicationMatched)
            .unwrap();
        waitset.attach(&writer, Some(&writer)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &writer);
        waitset.detach(&writer).unwrap();

        // Sync reader to writer.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        reader
            .set_status_mask(crate::Status::SubscriptionMatched)
            .unwrap();
        waitset.attach(&reader, Some(&reader)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &reader);
        waitset.detach(&reader).unwrap();

        let time = std::time::SystemTime::now().try_into().unwrap();
        for i in 0..4 {
            let sample = crate::tests::topic::Data {
                x: i,
                y: i.cast_signed() + 1,
                ..Default::default()
            };
            if sample.x.is_multiple_of(2) {
                if sample.x < 2 {
                    writer.write(&sample).unwrap();
                } else {
                    writer.write_with_timestamp(&sample, time).unwrap();
                }
            } else {
                let key = sample.as_key();
                if sample.x < 2 {
                    writer.write(&sample).unwrap();
                    writer.dispose(&key).unwrap();
                } else {
                    writer.write_with_timestamp(&sample, time).unwrap();
                    writer.dispose_with_timestamp(&key, time).unwrap();
                }
            }
        }

        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 4);

        for sample in samples {
            assert_eq!(
                *sample,
                crate::tests::topic::Data {
                    x: sample.x,
                    y: sample.x.cast_signed() + 1,
                    ..Default::default()
                }
            );
            if sample.x.is_multiple_of(2) {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Alive
                );
            } else {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Disposed
                );
            }
        }
    }

    #[test]
    fn test_writer_write_and_then_dispose_by_instance_handle() {
        use crate::entity::Entity;
        use crate::state;

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos
            .with_reliability(crate::qos::policy::Reliability::Reliable {
                max_blocking_time: std::time::Duration::from_millis(100).try_into().unwrap(),
            })
            .with_resource_limits(crate::qos::policy::ResourceLimits {
                max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                max_instances: crate::qos::policy::ResourceLimit::Limited(4),
                max_samples_per_instance: crate::qos::policy::ResourceLimit::Limited(1),
            });

        let reader = crate::Reader::builder(&topic)
            .with_qos(&qos)
            .build()
            .unwrap();

        let qos = qos.with_writer_data_lifecycle(crate::qos::policy::WriterDataLifecycle {
            autodispose_unregistered_instances: false,
        });
        let writer = Writer::builder(&topic).with_qos(&qos).build().unwrap();

        // Sync writer to reader.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        writer
            .set_status_mask(crate::Status::PublicationMatched)
            .unwrap();
        waitset.attach(&writer, Some(&writer)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &writer);
        waitset.detach(&writer).unwrap();

        // Sync reader to writer.
        let mut waitset = crate::WaitSet::new(&participant).unwrap();
        reader
            .set_status_mask(crate::Status::SubscriptionMatched)
            .unwrap();
        waitset.attach(&reader, Some(&reader)).unwrap();
        let result = waitset.wait(crate::Duration::INFINITE).unwrap();
        assert_eq!(result[0], &reader);
        waitset.detach(&reader).unwrap();

        let time = std::time::SystemTime::now().try_into().unwrap();
        for i in 0..4 {
            let sample = crate::tests::topic::Data {
                x: i,
                y: i.cast_signed() + 1,
                ..Default::default()
            };
            if sample.x.is_multiple_of(2) {
                if sample.x < 2 {
                    writer.write(&sample).unwrap();
                } else {
                    writer.write_with_timestamp(&sample, time).unwrap();
                }
            } else {
                let key = sample.as_key();
                if sample.x < 2 {
                    writer.write(&sample).unwrap();
                    let instance_handle = writer.lookup_instance(&key).unwrap();
                    writer.dispose_instance_by_handle(instance_handle).unwrap();
                } else {
                    writer.write_with_timestamp(&sample, time).unwrap();
                    let instance_handle = writer.lookup_instance(&key).unwrap();
                    writer
                        .dispose_instance_by_handle_with_timestamp(instance_handle, time)
                        .unwrap();
                }
            }
        }

        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 4);

        for sample in samples {
            assert_eq!(
                *sample,
                crate::tests::topic::Data {
                    x: sample.x,
                    y: sample.x.cast_signed() + 1,
                    ..Default::default()
                }
            );
            if sample.x % 2 == 0 {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Alive
                );
            } else {
                assert!(sample.is_sample());
                assert_eq!(
                    sample.info().state,
                    state::sample::Fresh | state::view::New | state::instance::Disposed
                );
            }
        }
    }
}
