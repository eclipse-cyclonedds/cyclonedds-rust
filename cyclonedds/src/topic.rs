use crate::internal::ffi;
use crate::internal::sertype::Sertype;
use crate::internal::traits::AsFfi;
use crate::{Participant, Result};

/// A typed communication channel.
///
/// A `Topic` binds a name to a data type [`T`](crate::Topicable) within a
/// [`Participant`](crate::Participant). [`Writers`](crate::Writer) and
/// [`Readers`](crate::Reader) are created against a topic and only match each
/// other when they share the same topic name and compatible type and
/// [`QoS`](crate::QoS).
///
/// Use [`Topic::new`] for simple construction or [`Topic::builder`] for
/// [`QoS`](crate::QoS) and [`listener`](crate::listener::TopicListener)
/// configuration.
#[derive(Debug)]
pub struct Topic<'domain, 'participant, T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom_type: std::marker::PhantomData<T>,
    phantom_participant: std::marker::PhantomData<&'participant Participant<'domain>>,
}

/// Builder for [`Topic<T>`] (accessible via [`Topic::builder`]).
#[derive(Debug)]
pub struct TopicBuilder<'domain, 'participant, 'qos, 'name, T>
where
    T: crate::Topicable,
{
    participant: &'participant Participant<'domain>,
    topic_name: &'name str,
    qos: Option<&'qos crate::QoS>,
    listener: Option<crate::TopicListener<T>>,
}

impl<'d, 'p, 'q, 'n, T> TopicBuilder<'d, 'p, 'q, 'n, T>
where
    T: crate::Topicable,
{
    /// Creates a new [`TopicBuilder`] for the given [`Participant`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::builder::TopicBuilder;
    /// use cyclonedds::{Domain, Participant};
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let domain = Domain::default();
    /// let participant = Participant::new(&domain)?;
    /// let topic_builder = TopicBuilder::<Data>::new(&participant, "MyTopic");
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn new(participant: &'p Participant<'d>, topic_name: &'n str) -> Self {
        Self {
            participant,
            topic_name,
            qos: None,
            listener: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this topic builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::builder::TopicBuilder;
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let qos = QoS::new().with_reliability(policy::Reliability::Reliable {
    ///     max_blocking_time: Duration::from_millis(100),
    /// });
    /// let topic_builder = TopicBuilder::<Data>::new(&participant, "MyTopic").with_qos(&qos);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Sets the [`Listener`](crate::Listener) on this topic builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::TopicListener;
    /// use cyclonedds::builder::TopicBuilder;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let participant_builder =
    ///     TopicBuilder::<Data>::new(&participant, "MyTopic").with_listener(TopicListener::new());
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: AsRef<crate::TopicListener<T>>,
    {
        self.listener = Some(listener.as_ref().clone());
        self
    }

    /// Builds the [`Topic`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the topic failed to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::builder::TopicBuilder;
    /// use cyclonedds::qos::policy;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let qos = QoS::new().with_durability(policy::Durability::TransientLocal);
    /// let topic = TopicBuilder::<Data>::new(&participant, "MyTopic")
    ///     .with_qos(&qos)
    ///     .build()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn build(self) -> Result<Topic<'d, 'p, T>> {
        let name = std::ffi::CString::new(self.topic_name)
            .map_err(|_err| crate::error::Error::BadParameter)?;
        let type_name = std::ffi::CString::new(T::dds_type_name().as_ref())
            .map_err(|_err| crate::error::Error::BadParameter)?;

        let mut sertype =
            std::mem::ManuallyDrop::new(Box::new(Sertype::<T>::new(&type_name, T::IS_KEYED)));

        self.listener
            .map(|listener| listener.as_ffi())
            .transpose()
            .and_then(|listener| {
                let inner = ffi::dds_create_topic(
                    self.participant.inner,
                    &name,
                    &mut &mut sertype.inner,
                    self.qos.map(|qos| &qos.inner),
                    listener.as_ref(),
                )
                .inspect_err(|_| {
                    ffi::ddsi_sertype_unref(&mut sertype.inner);
                })?;

                Ok(Topic {
                    inner,
                    phantom_type: std::marker::PhantomData,
                    phantom_participant: std::marker::PhantomData,
                })
            })
    }
}

impl<'d, 'p, T> Topic<'d, 'p, T>
where
    T: crate::Topicable,
{
    /// Creates a new `Topic` with the given name under `participant` using
    /// default [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::TopicListener).
    ///
    /// The topic name identifies the communication channel. Writers and
    /// readers match when they share the same name and compatible type.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if topic fails to create.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    /// use cyclonedds::Topic;
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(participant: &'p Participant<'d>, topic_name: &str) -> Result<Self> {
        Self::builder(participant, topic_name).build()
    }

    /// Returns a [`TopicBuilder`](crate::builder::TopicBuilder) for
    /// constructing a topic with custom [`QoS`](crate::QoS) or a
    /// [`listener`](crate::listener::TopicListener).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Topic;
    /// # use cyclonedds::{Domain, Participant};
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
    /// let topic = Topic::<Data>::builder(&participant, "MyTopic").build()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn builder<'q, 'n>(
        participant: &'p Participant<'d>,
        topic_name: &'n str,
    ) -> TopicBuilder<'d, 'p, 'q, 'n, T> {
        TopicBuilder::new(participant, topic_name)
    }

    pub(crate) const fn from_existing(
        inner: cyclonedds_sys::dds_entity_t,
    ) -> std::mem::ManuallyDrop<Self> {
        std::mem::ManuallyDrop::new(Self {
            inner,
            phantom_type: std::marker::PhantomData,
            phantom_participant: std::marker::PhantomData,
        })
    }

    /// Sets the [`TopicListener`](crate::TopicListener) on this topic,
    /// replacing any previously set listener.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the topic fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::TopicListener;
    /// # use cyclonedds::{Domain, Participant, Topic};
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
    /// let mut topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// topic.set_listener(TopicListener::new().with_inconsistent_topic(|_, status| {
    ///     println!("inconsistent topic: {status:?}");
    /// }))?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set_listener<L>(&mut self, listener: L) -> Result<()>
    where
        T: serde::ser::Serialize + serde::de::DeserializeOwned + std::clone::Clone + Default,
        L: AsRef<crate::TopicListener<T>>,
    {
        listener
            .as_ref()
            .as_ffi()
            .and_then(|listener| ffi::dds_set_listener(self.inner, Some(listener.inner)))
    }

    /// Removes the listener from this topic.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the topic fails to unset the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant, Topic};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    /// let mut topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// topic.unset_listener()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn unset_listener(&mut self) -> Result<()> {
        ffi::dds_set_listener(self.inner, None)?;
        Ok(())
    }

    /// Sets the [`TopicListener`](crate::TopicListener) on this topic,
    /// consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the topic fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::TopicListener;
    /// # use cyclonedds::{Domain, Participant, Topic};
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
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?.with_listener(TopicListener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn with_listener<L>(mut self, listener: L) -> Result<Self>
    where
        T: serde::ser::Serialize + serde::de::DeserializeOwned + std::clone::Clone + Default,
        L: AsRef<crate::TopicListener<T>>,
    {
        self.set_listener(listener).map(|_err| self)
    }
}

impl<T> Drop for Topic<'_, '_, T>
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

    #[test]
    fn test_topic_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let participant = Participant::new(&domain).unwrap();
        let _ = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let _ = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();
    }

    #[test]
    fn test_topic_create_with_invalid_names() {
        use crate::Topicable;

        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
        struct MockedTypeNameData;
        static MOCKED_NAME: std::sync::Mutex<&str> = std::sync::Mutex::new("");

        impl Topicable for MockedTypeNameData {
            type Key = ();

            fn from_key((): &Self::Key) -> Self {
                Self {}
            }

            fn as_key(&self) -> Self::Key {}

            fn dds_type_name() -> impl AsRef<str> {
                MOCKED_NAME.lock().unwrap().clone()
            }
        }

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let mut participant = crate::Participant::new(&domain).unwrap();

        let data = MockedTypeNameData {};
        let key = ();

        assert_eq!(data, MockedTypeNameData::from_key(&key));
        assert!(matches!(data.as_key(), ()));

        // (invalid type name, invalid topic name)
        *MOCKED_NAME.lock().unwrap() = "\0";
        let topic_name = "\0";

        let result = Topic::<crate::tests::topic::Data>::new(&participant, topic_name).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Topic::<crate::tests::topic::Data>::builder(&participant, topic_name)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        // (invalid type name, valid topic name)
        *MOCKED_NAME.lock().unwrap() = "\0";
        let topic_name = &crate::tests::topic::unique_name();

        let result = Topic::<MockedTypeNameData>::new(&participant, topic_name).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Topic::<MockedTypeNameData>::builder(&participant, topic_name)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        // (valid type name, invalid topic name)
        *MOCKED_NAME.lock().unwrap() = "ValidName";
        let topic_name = "\0";

        let result = Topic::<MockedTypeNameData>::new(&participant, topic_name).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Topic::<MockedTypeNameData>::builder(&participant, topic_name)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        // (valid type name, valid topic name) on invalid participant
        *MOCKED_NAME.lock().unwrap() = "ValidName";
        let topic_name = &crate::tests::topic::unique_name();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = Topic::<MockedTypeNameData>::new(&participant, topic_name).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Topic::<MockedTypeNameData>::builder(&participant, topic_name)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        participant.inner = participant_id;

        // (valid type name, valid topic name)
        *MOCKED_NAME.lock().unwrap() = "ValidName";
        let topic_name = &crate::tests::topic::unique_name();
        let _ = Topic::<MockedTypeNameData>::new(&participant, topic_name).unwrap();
        let _ = Topic::<MockedTypeNameData>::builder(&participant, topic_name)
            .build()
            .unwrap();
    }

    #[test]
    fn test_topic_create_with_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let topic_name = crate::tests::topic::unique_name();
        let mut participant = Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result =
            Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        participant.inner = participant_id;
    }

    #[test]
    fn test_topic_with_listener() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::TopicListener::new().with_inconsistent_topic(|_, _| ());

        let _ = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name)
            .unwrap()
            .with_listener(&listener)
            .unwrap();
        let _ = Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_listener(&listener)
            .build()
            .unwrap();

        let mut topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        topic.set_listener(&listener).unwrap();
        topic.unset_listener().unwrap();
    }

    #[test]
    fn test_topic_with_listener_on_invalid_topic() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::TopicListener::new();

        let mut topic = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let topic_id = topic.inner;
        topic.inner = 0;
        let result = topic.set_listener(&listener).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = topic.unset_listener().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        topic.inner = topic_id;
    }

    #[test]
    fn test_topic_create_from_existing() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_01 = Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let topic_02 = Topic::<crate::tests::topic::Data>::from_existing(topic_01.inner);
        assert_eq!(topic_01.inner, topic_02.inner);
    }
}
