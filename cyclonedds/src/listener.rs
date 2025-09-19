//! Listener types for reacting to [`status events`](crate::Status) on
//! [`entities`](crate::entity::Entity).
//!
//! Each entity type has a corresponding listener struct that holds optional
//! callbacks for the status events it can produce. Callbacks are plain function
//! pointers and are registered via chainable `with_*` methods.
//!
//! The listener structure mimics the DDS entity hierarchy. [`Listener`] is the
//! top-level type attached to a [`Participant`](crate::Participant) and
//! composes [`SubscriberListener`] and [`PublisherListener`]. Entity-specific
//! listeners ([`ReaderListener`] and [`WriterListener`]) are attached directly
//! to their respective entities.
//!
//! ```text
//! ╭───────────────────────╮          ╭─────────────────────────────────────╮
//! │        Entity         │          │               Listener              │
//! ╰───────────────────────╯          ╰─────────────────────────────────────╯
//!  
//!     Domain
//!       │
//!  Participant ··················································· Listener
//!       ├─ Topic<T> ······························ TopicListener<T>  ─┤
//!       ├─ Subscriber ··························· SubscriberListener ─┤
//!       │     └─ Reader<T> ··········· ReaderListener<T> ───┘         │
//!       └─ Publisher ····························· PublisherListener ─┘
//!            └─ Writer<T> ············ WriterListener<T> ───┘
//! ```
//!
//! Listeners can be set at any level of the entity hierarchy. A listener set on
//! a [`Participant`](crate::Participant) will have its callbacks inherited by
//! child entities of that participant.
//!
//! Alternatively, a higher-level listener can also be passed directly to the
//! child entity's builder (as each listener type implements [`AsRef`] for the
//! listener types below it in the hierarchy). As a result, a single
//! [`Listener`] can be reused across multiple entity builders without
//! constructing separate listeners for each level.
//!
//! ```
//! use cyclonedds::Listener;
//! use cyclonedds::{Domain, Participant, Subscriber};
//!
//! let domain = Domain::default();
//!
//! // Create a participant listener with the subscriber callbacks configured.
//! let listener = Listener::new().with_subscriber(|s| {
//!     s.with_data_on_readers(|subscriber| {
//!         println!("{subscriber:?} has data");
//!     })
//! });
//!
//! // Create a participant with the listener.
//! let participant = Participant::builder(&domain)
//!     .with_listener(&listener)
//!     .build()?;
//!
//! // Subscribers created under the participant will inherit the `data_on_readers`
//! // callback.
//! let subscriber = Subscriber::new(&participant)?;
//!
//! // This subscriber is explicitly created with the subscriber portion of the
//! // `listener`.
//! let subscriber = Subscriber::builder(&participant)
//!     .with_listener(&listener)
//!     .build()?;
//!
//! # Ok::<_, cyclonedds::Error>(())
//! ```
//!
//! Each callback fires when its corresponding [`Status`](crate::Status)
//! condition is triggered. Most callbacks receive a status value from the
//! [`status`](crate::status) module carrying event-specific detail such as
//! counts and last-instance handles.
//!
//! # Warning
//!
//! <div class="warning">
//!
//! **Unstable:** The full DDS listener hierarchy, where [`TopicListener<T>`]
//! composes under [`Listener`] and [`ReaderListener<T>`] and
//! [`WriterListener<T>`] compose under [`SubscriberListener`] and
//! [`PublisherListener`], respectively, is not yet implemented.
//!
//! The [`Listener`], [`SubscriberListener`], and [`PublisherListener`] may
//! propagate to many [`Topic<T>`](crate::Topic), [`Reader<T>`](crate::Reader),
//! and [`Writer<T>`](crate::Writer) that all have different types for `<T>`. As
//! a result, one of two obvious solutions presents itself:
//!
//! - Allow these higher-level types to only have callbacks of effectively
//!   [`std::any::Any`] and require the callback to attempt to convert. This
//!   maps the most correctly onto how the API is designed in the specification
//!   but would greatly complicate the internal dispatching of these listeners.
//!
//! - Maintain a typed registry of all the different types of callbacks that are
//!   attached on the higher-level untyped subscribers and then add code to
//!   check if the event that fired corresponds to a type whose callback was
//!   registered. This would work but introduces semantics that do not match the
//!   other DDS implementations.
//!
//! </div>
//!
//! # Examples
//!
//! ```
//! use cyclonedds::entity::Entity;
//! use cyclonedds::{Reader, Topic, Writer};
//! use cyclonedds::{ReaderListener, TopicListener, WriterListener};
//! # #[derive(
//! #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
//! # )]
//! # struct Data {
//! #     x: i32,
//! # }
//! # let domain = cyclonedds::Domain::default();
//! # let participant = cyclonedds::Participant::new(&domain)?;
//!
//! let topic = Topic::<Data>::builder(&participant, "Example")
//!     .with_listener(
//!         TopicListener::new().with_inconsistent_topic(|topic, inconsistent_topic| {
//!             println!(
//!                 "{topic:?} inconsistent topic: {} just encountered, {} encountered in total",
//!                 inconsistent_topic.total.delta, inconsistent_topic.total.count
//!             )
//!         }),
//!     )
//!     .build()?;
//!
//! let reader = Reader::builder(&topic)
//!     .with_listener(
//!         ReaderListener::new()
//!             .with_subscription_matched(|reader, subscription_matched| {
//!                 println!("{reader:?} had a subscription match: {subscription_matched:?}")
//!             })
//!             .with_sample_lost(|reader, sample_lost| {
//!                 println!(
//!                     "{reader:?} lost samples: {} just lost, {} lost in total",
//!                     sample_lost.total.delta, sample_lost.total.count
//!                 )
//!             }),
//!     )
//!     .build()?;
//!
//! let writer = Writer::builder(&topic)
//!     .with_listener(
//!         WriterListener::new()
//!             .with_publication_matched(|writer, publication_matched| {
//!                 println!("{writer:?} has a publication match: {publication_matched:?}")
//!             })
//!             .with_liveliness_lost(|writer, liveliness_lost| {
//!                 println!(
//!                     "{writer:?} liveliness lost: {} just lost, {} lost in total",
//!                     liveliness_lost.total.delta, liveliness_lost.total.count
//!                 )
//!             }),
//!     )
//!     .build()?;
//! # Ok::<_, cyclonedds::Error>(())
//! ```

use crate::Result;
use crate::internal::ffi;
use crate::internal::traits::AsFfi;
use crate::status::{
    InconsistentTopic, LivelinessChanged, LivelinessLost, OfferedDeadlineMissed,
    OfferedIncompatibleQoS, PublicationMatched, RequestedDeadlineMissed, RequestedIncompatibleQoS,
    SampleLost, SampleRejected, SubscriptionMatched,
};

/// Listener attached to a [`Participant`](crate::Participant).
///
/// In the DDS entity hierarchy this composes [`SubscriberListener`],
/// [`PublisherListener`], and [`TopicListener`]. When attached to a
/// participant, entities created under it inherit any of the configured
/// callbacks that apply to that entity type.
///
/// # Examples
///
/// ```
/// use cyclonedds::{Domain, Listener, Participant, Subscriber};
///
/// let domain = Domain::default();
/// let listener = Listener::new().with_subscriber(|subscriber_listener| {
///     subscriber_listener
///         .with_data_on_readers(|subscriber| println!("{subscriber:?} has data on readers"))
/// });
/// let participant = Participant::builder(&domain)
///     .with_listener(&listener)
///     .build()?;
///
/// // This subscriber inherits the callbacks set on the `participant` via the `listener`.
/// let subscriber = Subscriber::new(&participant)?;
///
/// // This subscriber will have the subscriber subset associated with the `listener` directly
/// // applied to it.
/// let subscriber = Subscriber::builder(&participant)
///     .with_listener(&listener)
///     .build()?;
/// # Ok::<_, cyclonedds::Error>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Listener {
    // topic: TopicListener<T>,
    subscriber: SubscriberListener,
    publisher: PublisherListener,
}

/// Listener attached to a [`Topic<T>`](crate::Topic<T>).
#[derive(Debug, Clone, Copy)]
pub struct TopicListener<T>
where
    T: crate::Topicable,
{
    inconsistent_topic: Option<fn(&crate::Topic<'_, '_, T>, InconsistentTopic)>,
}

/// Listener attached to a [`Subscriber`](crate::Subscriber).
///
/// <div class="warning">
///
/// Currently [`SubscriberListener`] is missing its configuration for composing
/// a [`ReaderListener<T>`] under this non-generic type. See the [module-level
/// warning](crate::listener#warning) for more detail.
///
/// </div>
#[derive(Debug, Default, Clone, Copy)]
pub struct SubscriberListener {
    data_on_readers: Option<fn(&crate::Subscriber<'_, '_>)>,
    // ///
    // pub reader: ReaderListener<T>,
}

/// Listener attached to a [`Reader<T>`](crate::Reader<T>).
#[derive(Debug, Clone, Copy)]
pub struct ReaderListener<T>
where
    T: crate::Topicable,
{
    sample_lost: Option<fn(&crate::Reader<'_, '_, '_, T>, SampleLost)>,
    data_available: Option<fn(&crate::Reader<'_, '_, '_, T>)>,
    sample_rejected: Option<fn(&crate::Reader<'_, '_, '_, T>, SampleRejected)>,
    liveliness_changed: Option<fn(&crate::Reader<'_, '_, '_, T>, LivelinessChanged)>,
    requested_deadline_missed: Option<fn(&crate::Reader<'_, '_, '_, T>, RequestedDeadlineMissed)>,
    requested_incompatible_qos: Option<fn(&crate::Reader<'_, '_, '_, T>, RequestedIncompatibleQoS)>,
    subscription_matched: Option<fn(&crate::Reader<'_, '_, '_, T>, SubscriptionMatched)>,
}

/// Listener attached to a [`Publisher`](crate::Publisher).
///
/// <div class="warning">
///
/// Currently [`PublisherListener`] has no registered callbacks pending a
/// solution for composing [`WriterListener<T>`] under this non-generic type.
/// See the [module-level warning](crate::listener#warning) for more detail.
///
/// </div>
#[derive(Debug, Default, Clone, Copy)]
pub struct PublisherListener {
    // ///
    // pub writer: WriterListener<T>,
}

/// Listener attached to a [`Writer<T>`](crate::Writer<T>).
#[derive(Debug, Clone, Copy)]
pub struct WriterListener<T>
where
    T: crate::Topicable,
{
    liveliness_lost: Option<fn(&crate::Writer<'_, '_, '_, T>, LivelinessLost)>,
    offered_deadline_missed: Option<fn(&crate::Writer<'_, '_, '_, T>, OfferedDeadlineMissed)>,
    offered_incompatible_qos: Option<fn(&crate::Writer<'_, '_, '_, T>, OfferedIncompatibleQoS)>,
    publication_matched: Option<fn(&crate::Writer<'_, '_, '_, T>, PublicationMatched)>,
}

impl<T> Default for TopicListener<T>
where
    T: crate::Topicable,
{
    fn default() -> Self {
        Self {
            inconsistent_topic: Option::default(),
        }
    }
}

impl<T> Default for ReaderListener<T>
where
    T: crate::Topicable,
{
    fn default() -> Self {
        Self {
            sample_lost: Option::default(),
            data_available: Option::default(),
            sample_rejected: Option::default(),
            liveliness_changed: Option::default(),
            requested_deadline_missed: Option::default(),
            requested_incompatible_qos: Option::default(),
            subscription_matched: Option::default(),
        }
    }
}

impl<T> Default for WriterListener<T>
where
    T: crate::Topicable,
{
    fn default() -> Self {
        Self {
            liveliness_lost: Option::default(),
            offered_deadline_missed: Option::default(),
            offered_incompatible_qos: Option::default(),
            publication_matched: Option::default(),
        }
    }
}

impl Listener {
    /// Creates a new [`Listener`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Listener;
    ///
    /// let listener = Listener::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ///
    // pub fn with_topic(mut self, setter: fn(TopicListener<T>) -> TopicListener<T>)
    // -> Self {     self.topic = setter(self.topic);
    //     self
    // }

    /// Configures the [`SubscriberListener`] via a setter callback.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Listener;
    ///
    /// let listener = Listener::new().with_subscriber(|s| {
    ///     s.with_data_on_readers(|subscriber| {
    ///         println!("data available on a reader");
    ///     })
    /// });
    /// ```
    #[must_use]
    pub fn with_subscriber(mut self, setter: fn(SubscriberListener) -> SubscriberListener) -> Self {
        self.subscriber = setter(self.subscriber);
        self
    }

    /// Configures the [`PublisherListener`] via a setter callback.
    ///
    /// # Examples
    ///
    /// <div class="warning">
    ///
    /// This example does not compile because the [`PublisherListener`] does not
    /// have its `with_writer::<T>` setter yet. This is due to the fact that
    /// the higher-level listeners are untyped in `<T>` but the lower-level
    /// listeners are typed in `<T>` and a solution for crossing this boundary
    /// still needs to be worked out.
    ///
    /// See the [module-level warning](crate::listener#warning) for more detail.
    ///
    /// </div>
    ///
    /// ```ignore
    /// use cyclonedds::Listener;
    ///
    /// let listener = Listener::new().with_publisher(|p| {
    ///     p.with_writer(|w| {
    ///         w.with_publication_matched(|writer, publication_matched| {
    ///             println!("{writer:?} has publication match: {publication_matched:?}")
    ///         })
    ///     })
    /// });
    /// ```
    #[must_use]
    pub fn with_publisher(mut self, setter: fn(PublisherListener) -> PublisherListener) -> Self {
        self.publisher = setter(self.publisher);
        self
    }

    #[inline]
    pub(crate) fn apply_listener_ffi(self, listener: &mut ffi::Listener) {
        // self.topic.apply_listener_ffi(listener);
        self.subscriber.apply_listener_ffi(listener);
        self.publisher.apply_listener_ffi(listener);
    }
}

impl AsFfi for Listener {
    type Target<'a> = Result<ffi::Listener>;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl<T> TopicListener<T>
where
    T: crate::Topicable,
{
    /// Creates a new [`TopicListener<T>`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::TopicListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = TopicListener::<Data>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a callback for the
    /// [`InconsistentTopic` status event](crate::Status::InconsistentTopic).
    ///
    /// The callback receives an
    /// [`InconsistentTopic` metadata struct](InconsistentTopic).
    ///
    /// Fired when a remote topic is discovered with the same name but an
    /// incompatible type or [`QoS`](crate::QoS).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::TopicListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener =
    ///     TopicListener::<Data>::new().with_inconsistent_topic(|topic, inconsistent_topic| {
    ///         println!("inconsistent topic: {inconsistent_topic:?}");
    ///     });
    /// ```
    #[must_use]
    pub fn with_inconsistent_topic(
        mut self,
        callback: fn(&crate::Topic<'_, '_, T>, InconsistentTopic),
    ) -> Self {
        self.inconsistent_topic = Some(callback);
        self
    }

    #[inline]
    pub(crate) fn apply_listener_ffi(&self, listener: &mut ffi::Listener) {
        if let Some(callback) = self.inconsistent_topic {
            ffi::dds_listener_set_inconsistent_topic(listener, callback);
        }
    }
}

impl<T> AsFfi for TopicListener<T>
where
    T: crate::Topicable,
{
    type Target<'a>
        = Result<ffi::Listener>
    where
        T: 'a;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl SubscriberListener {
    /// Creates a new [`SubscriberListener`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::SubscriberListener;
    ///
    /// let listener = SubscriberListener::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ///
    // pub fn with_reader(mut self, setter: fn(ReaderListener<T>) ->
    // ReaderListener<T>) -> Self {     self.reader = setter(self.reader);
    //     self
    // }

    /// Sets a callback for the [`DataOnReaders` status
    /// event](crate::Status::DataOnReaders).
    ///
    /// Fired when new data is available on one or more readers belonging to
    /// this subscriber.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::SubscriberListener;
    ///
    /// let listener = SubscriberListener::new().with_data_on_readers(|subscriber| {
    ///     println!("data available on {subscriber:?}");
    /// });
    /// ```
    #[must_use]
    pub fn with_data_on_readers(mut self, callback: fn(&crate::Subscriber<'_, '_>)) -> Self {
        self.data_on_readers = Some(callback);
        self
    }

    #[inline]
    pub(crate) fn apply_listener_ffi(self, listener: &mut ffi::Listener) {
        if let Some(callback) = self.data_on_readers {
            ffi::dds_listener_set_data_on_readers(listener, callback);
        }
        // self.reader.apply_listener_ffi(listener);
    }
}

impl AsFfi for SubscriberListener {
    type Target<'a> = Result<ffi::Listener>;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl PublisherListener {
    /// Creates a new [`PublisherListener`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::PublisherListener;
    ///
    /// let listener = PublisherListener::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ///
    // pub fn with_writer(mut self, setter: fn(WriterListener<T>) ->
    // WriterListener<T>) -> Self {     self.writer = setter(self.writer);
    //     self
    // }

    #[inline]
    pub(crate) const fn apply_listener_ffi(self, listener: &mut ffi::Listener) {
        let _ = self;
        let _ = listener;
        // self.writer.apply_listener_ffi(listener);
    }
}

impl AsFfi for PublisherListener {
    type Target<'a> = Result<ffi::Listener>;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl<T> ReaderListener<T>
where
    T: crate::Topicable,
{
    /// Creates a new [`ReaderListener<T>`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = ReaderListener::<Data>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a callback for the [`SampleLost` status
    /// event](crate::Status::SampleLost).
    ///
    /// The callback receives a [`SampleLost` metadata struct](SampleLost).
    ///
    /// Fired when a sample is lost, meaning it was never received by this
    /// reader due to resource limits or [`QoS`](crate::QoS) constraints.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = ReaderListener::<Data>::new().with_sample_lost(|reader, sample_lost| {
    ///     println!("samples lost: {}", sample_lost.total.count);
    /// });
    /// ```
    #[must_use]
    pub fn with_sample_lost(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, SampleLost),
    ) -> Self {
        self.sample_lost = Some(callback);
        self
    }

    /// Sets a callback for the [`DataAvailable` status
    /// event](crate::Status::DataAvailable).
    ///
    /// Fired when new data is available to be [`peeked`](crate::Reader::peek),
    /// [`read`](crate::Reader::read), or [`taken`](crate::Reader::take) from
    /// this reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = ReaderListener::<Data>::new().with_data_available(|reader| {
    ///     println!("data available on {reader:?}");
    /// });
    /// ```
    #[must_use]
    pub fn with_data_available(mut self, callback: fn(&crate::Reader<'_, '_, '_, T>)) -> Self {
        self.data_available = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`SampleRejected` status event](crate::Status::SampleRejected).
    ///
    /// The callback receives a [`SampleRejected` metadata
    /// struct](SampleRejected).
    ///
    /// Fired when an incoming sample is rejected due to
    /// [`ResourceLimits`](crate::qos::policy::ResourceLimits).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    /// let listener = ReaderListener::<Data>::new().with_sample_rejected(|reader, status| {
    ///     println!("sample rejected: {status:?}");
    /// });
    /// ```
    #[must_use]
    pub fn with_sample_rejected(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, SampleRejected),
    ) -> Self {
        self.sample_rejected = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`LivelinessChanged` status event](crate::Status::LivelinessChanged).
    ///
    /// The callback receives a
    /// [`LivelinessChanged` metadata struct](LivelinessChanged).
    ///
    /// Fired when the [`Liveliness`](crate::qos::policy::Liveliness) of a
    /// matched writer changes, i.e. a writer becomes active or inactive.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener =
    ///     ReaderListener::<Data>::new().with_liveliness_changed(|reader, liveliness_changed| {
    ///         println!("active writers: {}", liveliness_changed.alive.count);
    ///     });
    /// ```
    #[must_use]
    pub fn with_liveliness_changed(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, LivelinessChanged),
    ) -> Self {
        self.liveliness_changed = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`RequestedDeadlineMissed` status
    /// event](crate::Status::RequestedDeadlineMissed).
    ///
    /// The callback receives a
    /// [`RequestedDeadlineMissed` metadata struct](RequestedDeadlineMissed).
    ///
    /// Fired when a sample is not received within the
    /// [`Deadline`](crate::qos::policy::Deadline) period offered by a matched
    /// writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = ReaderListener::<Data>::new().with_requested_deadline_missed(
    ///     |reader, requested_deadline_missed| {
    ///         println!("deadline missed: {}", requested_deadline_missed.total.count);
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn with_requested_deadline_missed(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, RequestedDeadlineMissed),
    ) -> Self {
        self.requested_deadline_missed = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`RequestedIncompatibleQoS` status
    /// event](crate::Status::RequestedIncompatibleQoS).
    ///
    /// The callback receives a
    /// [`RequestedIncompatibleQoS` metadata struct](RequestedIncompatibleQoS).
    ///
    /// Fired when a writer is discovered whose offered [`QoS`](crate::QoS) is
    /// incompatible with this reader's requested [`QoS`](crate::QoS).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = ReaderListener::<Data>::new().with_requested_incompatible_qos(
    ///     |reader, requested_incompatible_qos| {
    ///         println!("incompatible QoS: {requested_incompatible_qos:?}");
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn with_requested_incompatible_qos(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, RequestedIncompatibleQoS),
    ) -> Self {
        self.requested_incompatible_qos = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`SubscriptionMatched` status event](crate::Status::SubscriptionMatched)
    /// status event.
    ///
    /// The callback receives a
    /// [`SubscriptionMatched` metadata struct](SubscriptionMatched).
    ///
    /// Fired when a writer matching this reader's topic and [`QoS`](crate::QoS)
    /// is discovered or lost.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::ReaderListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener =
    ///     ReaderListener::<Data>::new().with_subscription_matched(|reader, subscription_matched| {
    ///         println!("matched writers: {}", subscription_matched.current.count);
    ///     });
    /// ```
    #[must_use]
    pub fn with_subscription_matched(
        mut self,
        callback: fn(&crate::Reader<'_, '_, '_, T>, SubscriptionMatched),
    ) -> Self {
        self.subscription_matched = Some(callback);
        self
    }

    #[inline]
    pub(crate) fn apply_listener_ffi(&self, listener: &mut ffi::Listener) {
        if let Some(callback) = self.sample_lost {
            ffi::dds_listener_set_sample_lost(listener, callback);
        }
        if let Some(callback) = self.data_available {
            ffi::dds_listener_set_data_available(listener, callback);
        }
        if let Some(callback) = self.sample_rejected {
            ffi::dds_listener_set_sample_rejected(listener, callback);
        }
        if let Some(callback) = self.liveliness_changed {
            ffi::dds_listener_set_liveliness_changed(listener, callback);
        }
        if let Some(callback) = self.requested_deadline_missed {
            ffi::dds_listener_set_requested_deadline_missed(listener, callback);
        }
        if let Some(callback) = self.requested_incompatible_qos {
            ffi::dds_listener_set_requested_incompatible_qos(listener, callback);
        }
        if let Some(callback) = self.subscription_matched {
            ffi::dds_listener_set_subscription_matched(listener, callback);
        }
    }
}

impl<T> AsFfi for ReaderListener<T>
where
    T: crate::Topicable,
{
    type Target<'a>
        = Result<ffi::Listener>
    where
        T: 'a;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl<T> WriterListener<T>
where
    T: crate::Topicable,
{
    /// Creates a new [`WriterListener<T>`] with no callbacks registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::TopicListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = TopicListener::<Data>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a callback for the
    /// [`LivelinessLost` status event](crate::Status::LivelinessLost).
    ///
    /// The callback receives a [`LivelinessLost` metadata
    /// struct](LivelinessLost).
    ///
    /// Fired when the writer fails to meet its
    /// [`Liveliness`](crate::qos::policy::Liveliness) policy and is considered
    /// inactive by matched readers.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::WriterListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = WriterListener::<Data>::new().with_liveliness_lost(|writer, liveliness_lost| {
    ///     println!(
    ///         "{writer:?} liveliness lost: {}",
    ///         liveliness_lost.total.count
    ///     );
    /// });
    /// ```
    #[must_use]
    pub fn with_liveliness_lost(
        mut self,
        callback: fn(&crate::Writer<'_, '_, '_, T>, LivelinessLost),
    ) -> Self {
        self.liveliness_lost = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`OfferedDeadlineMissed` status
    /// event](crate::Status::OfferedDeadlineMissed) status event.
    ///
    /// The callback receives an
    /// [`OfferedDeadlineMissed` metadata struct](OfferedDeadlineMissed).
    ///
    /// Fired when the writer fails to write a new sample within its offered
    /// [`Deadline`](crate::qos::policy::Deadline) period for one or more
    /// instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::WriterListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = WriterListener::<Data>::new().with_offered_deadline_missed(
    ///     |writer, offered_deadline_missed| {
    ///         println!(
    ///             "{writer:?} deadline missed: {}",
    ///             offered_deadline_missed.total.count
    ///         );
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn with_offered_deadline_missed(
        mut self,
        callback: fn(&crate::Writer<'_, '_, '_, T>, OfferedDeadlineMissed),
    ) -> Self {
        self.offered_deadline_missed = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`OfferedIncompatibleQoS` status
    /// event](crate::Status::OfferedIncompatibleQoS) status event.
    ///
    /// The callback receives an
    /// [`OfferedIncompatibleQoS` metadata struct](OfferedIncompatibleQoS).
    ///
    /// Fired when a reader is discovered whose requested [`QoS`](crate::QoS) is
    /// incompatible with this writer's offered [`QoS`](crate::QoS).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::WriterListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = WriterListener::<Data>::new().with_offered_incompatible_qos(
    ///     |writer, offered_incompatible_qos| {
    ///         println!("{writer:?} discovered incompatible QoS: {offered_incompatible_qos:?}");
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn with_offered_incompatible_qos(
        mut self,
        callback: fn(&crate::Writer<'_, '_, '_, T>, OfferedIncompatibleQoS),
    ) -> Self {
        self.offered_incompatible_qos = Some(callback);
        self
    }

    /// Sets a callback for the
    /// [`PublicationMatched` status event](crate::Status::PublicationMatched).
    ///
    /// The callback receives a
    /// [`PublicationMatched` metadata struct](PublicationMatched).
    ///
    /// Fired when a reader matching this writer's topic and [`QoS`](crate::QoS)
    /// is discovered.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::listener::WriterListener;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// # }
    ///
    /// let listener = WriterListener::<Data>::new().with_publication_matched(|writer, status| {
    ///     println!("{writer:?} matched readers: {}", status.current.count);
    /// });
    /// ```
    #[must_use]
    pub fn with_publication_matched(
        mut self,
        callback: fn(&crate::Writer<'_, '_, '_, T>, PublicationMatched),
    ) -> Self
    where
        T: crate::Topicable,
    {
        self.publication_matched = Some(callback);
        self
    }

    #[inline]
    pub(crate) fn apply_listener_ffi(&self, listener: &mut ffi::Listener) {
        if let Some(callback) = self.liveliness_lost {
            ffi::dds_listener_set_liveliness_lost(listener, callback);
        }
        if let Some(callback) = self.offered_deadline_missed {
            ffi::dds_listener_set_offered_deadline_missed(listener, callback);
        }
        if let Some(callback) = self.offered_incompatible_qos {
            ffi::dds_listener_set_offered_incompatible_qos(listener, callback);
        }
        if let Some(callback) = self.publication_matched {
            ffi::dds_listener_set_publication_matched(listener, callback);
        }
    }
}

impl<T> AsFfi for WriterListener<T>
where
    T: crate::Topicable,
{
    type Target<'a>
        = Result<ffi::Listener>
    where
        T: 'a;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        ffi::Listener::new().map(|mut listener| {
            self.apply_listener_ffi(&mut listener);
            listener
        })
    }
}

impl<T> AsRef<ReaderListener<T>> for ReaderListener<T>
where
    T: crate::Topicable,
{
    fn as_ref(&self) -> &ReaderListener<T> {
        self
    }
}
impl<T> AsRef<WriterListener<T>> for WriterListener<T>
where
    T: crate::Topicable,
{
    fn as_ref(&self) -> &WriterListener<T> {
        self
    }
}
impl AsRef<SubscriberListener> for SubscriberListener {
    fn as_ref(&self) -> &SubscriberListener {
        self
    }
}
impl AsRef<PublisherListener> for PublisherListener {
    fn as_ref(&self) -> &PublisherListener {
        self
    }
}
impl<T> AsRef<TopicListener<T>> for TopicListener<T>
where
    T: crate::Topicable,
{
    fn as_ref(&self) -> &TopicListener<T> {
        self
    }
}
impl AsRef<Listener> for Listener {
    fn as_ref(&self) -> &Listener {
        self
    }
}

// impl<T> AsRef<ReaderListener<T>> for Listener<T> {
//     fn as_ref(&self) -> &ReaderListener<T> {
//         &self.subscriber.reader
//     }
// }
// impl<T> AsRef<WriterListener<T>> for Listener<T> {
//     fn as_ref(&self) -> &WriterListener<T> {
//         &self.publisher.writer
//     }
// }
impl AsRef<SubscriberListener> for Listener {
    fn as_ref(&self) -> &SubscriberListener {
        &self.subscriber
    }
}
impl AsRef<PublisherListener> for Listener {
    fn as_ref(&self) -> &PublisherListener {
        &self.publisher
    }
}
// impl<T> AsRef<TopicListener<T>> for Listener<T> {
//     fn as_ref(&self) -> &TopicListener<T> {
//         &self.topic
//     }
// }

// impl<T> AsRef<ReaderListener<T>> for SubscriberListener<T> {
//     fn as_ref(&self) -> &ReaderListener<T> {
//         &self.reader
//     }
// }
// impl<T> AsRef<WriterListener<T>> for PublisherListener<T> {
//     fn as_ref(&self) -> &WriterListener<T> {
//         &self.writer
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Topicable;

    fn receive_listener<L>(listener: L)
    where
        L: AsRef<Listener>,
    {
        let _ = listener.as_ref();
    }

    fn receive_topic_listener<L, T>(listener: L)
    where
        L: AsRef<TopicListener<T>>,
        T: crate::Topicable,
    {
        let _ = listener.as_ref();
    }

    fn receive_subscriber_listener<L>(listener: L)
    where
        L: AsRef<SubscriberListener>,
    {
        let _ = listener.as_ref();
    }

    fn receive_publisher_listener<L>(listener: L)
    where
        L: AsRef<PublisherListener>,
    {
        let _ = listener.as_ref();
    }

    fn receive_reader_listener<L, T>(listener: L)
    where
        L: AsRef<ReaderListener<T>>,
        T: crate::Topicable,
    {
        let _ = listener.as_ref();
    }

    fn receive_writer_listener<L, T>(listener: L)
    where
        L: AsRef<WriterListener<T>>,
        T: crate::Topicable,
    {
        let _ = listener.as_ref();
    }

    #[test]
    fn test_listener_create() {
        let listener = Listener::new()
            // .with_topic(|topic| topic.with_inconsistent_topic(|_, _| ()))
            .with_subscriber(|subscriber| {
                subscriber.with_data_on_readers(|_| ())
                // .with_reader(|reader| {
                //     reader
                //         .with_data_available(|_| ())
                //         .with_liveliness_changed(|_, _| ())
                //         .with_requested_deadline_missed(|_, _| ())
                //         .with_requested_incompatible_qos(|_, _| ())
                //         .with_sample_lost(|_, _| ())
                //         .with_sample_rejected(|_, _| ())
                //         .with_subscription_matched(|_, _| ())
                // })
            })
            .with_publisher(|publisher| {
                publisher
                //     .with_writer(|writer| {
                //     writer
                //         .with_liveliness_lost(|_, _| ())
                //         .with_offered_deadline_missed(|_, _| ())
                //         .with_offered_incompatible_qos(|_, _| ())
                //         .with_publication_matched(|_, _| ())
                // })
            });
        let topic_listener =
            TopicListener::<crate::tests::topic::Data>::new().with_inconsistent_topic(|_, _| ());
        let subscriber_listener = SubscriberListener::new()
            .with_data_on_readers(|_| ())
        // .with_reader(|reader| {
        //     reader
        //         .with_data_available(|_| ())
        //         .with_liveliness_changed(|_, _| ())
        //         .with_requested_deadline_missed(|_, _| ())
        //         .with_requested_incompatible_qos(|_, _| ())
        //         .with_sample_lost(|_, _| ())
        //         .with_sample_rejected(|_, _| ())
        //         .with_subscription_matched(|_, _| ())
        // })
            ;
        let publisher_listener =
            PublisherListener::new()
        // .with_writer(|writer| {
        //     writer
        //         .with_liveliness_lost(|_, _| ())
        //         .with_offered_deadline_missed(|_, _| ())
        //         .with_offered_incompatible_qos(|_, _| ())
        //         .with_publication_matched(|_, _| ())
        // })
            ;
        let reader_listener = ReaderListener::<crate::tests::topic::Data>::new()
            .with_data_available(|_| ())
            .with_liveliness_changed(|_, _| ())
            .with_requested_deadline_missed(|_, _| ())
            .with_requested_incompatible_qos(|_, _| ())
            .with_sample_lost(|_, _| ())
            .with_sample_rejected(|_, _| ())
            .with_subscription_matched(|_, _| ());
        let writer_listener = WriterListener::<crate::tests::topic::Data>::new()
            .with_liveliness_lost(|_, _| ())
            .with_offered_deadline_missed(|_, _| ())
            .with_offered_incompatible_qos(|_, _| ())
            .with_publication_matched(|_, _| ());

        receive_listener(listener);

        receive_topic_listener(&topic_listener);
        // receive_topic_listener(&listener);

        receive_subscriber_listener(subscriber_listener);
        receive_subscriber_listener(listener);

        receive_publisher_listener(publisher_listener);
        receive_publisher_listener(listener);

        receive_reader_listener(&reader_listener);
        // receive_reader_listener(&subscriber_listener);
        // receive_reader_listener(&listener);

        receive_writer_listener(&writer_listener);
        // receive_writer_listener(&publisher_listener);
        // receive_writer_listener(&listener);
    }

    #[test]
    fn test_subscriber_listener_callbacks() {
        #[derive(Debug, PartialEq)]
        struct Triggered {
            data_on_readers: u32,
        }

        static TRIGGERED: std::sync::Mutex<Triggered> =
            std::sync::Mutex::new(Triggered { data_on_readers: 0 });

        let domain_id = crate::tests::domain::unique_id();
        let topic_name = crate::tests::topic::unique_name();
        let domain = crate::Domain::new(domain_id).unwrap();

        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let subscriber = crate::Subscriber::builder(&participant)
            .with_listener(
                crate::SubscriberListener::new().with_data_on_readers(|_subscriber| {
                    TRIGGERED.lock().unwrap().data_on_readers += 1;
                }),
            )
            .build()
            .unwrap();
        let reader = crate::Reader::builder(&topic)
            .with_subscriber(&subscriber)
            .build()
            .unwrap();
        let writer = crate::Writer::new(&topic).unwrap();

        let sample = crate::tests::topic::Data::default();
        writer.write(&sample).unwrap();

        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 1);

        assert_eq!(*samples[0], sample);

        assert_eq!(*TRIGGERED.lock().unwrap(), Triggered { data_on_readers: 1 });
    }

    #[test]
    fn test_publisher_listener_callbacks() {
        let domain_id = crate::tests::domain::unique_id();
        let topic_name = crate::tests::topic::unique_name();
        let domain = crate::Domain::new(domain_id).unwrap();

        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let publisher = crate::Publisher::builder(&participant)
            .with_listener(crate::PublisherListener::new())
            .build()
            .unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::builder(&topic)
            .with_publisher(&publisher)
            .build()
            .unwrap();

        let sample = crate::tests::topic::Data::default();
        writer.write(&sample).unwrap();

        let samples = reader.read().unwrap();
        assert_eq!(samples.len(), 1);

        assert_eq!(*samples[0], sample);
    }

    #[test]
    fn test_reader_listener_callbacks() {
        #[derive(Debug, PartialEq)]
        struct Triggered {
            requested_incompatible_qos: u32,
            requested_deadline_missed: bool,
            sample_rejected: u32,
            data_available: u32,
            subscription_matched: u32,
            liveliness_changed: u32,
            sample_lost: u32,
        }

        static TRIGGERED: std::sync::Mutex<Triggered> = std::sync::Mutex::new(Triggered {
            requested_incompatible_qos: 0,
            requested_deadline_missed: false,
            sample_rejected: 0,
            data_available: 0,
            subscription_matched: 0,
            liveliness_changed: 0,
            sample_lost: 0,
        });

        let domain_id = crate::tests::domain::unique_id();
        let topic_name = crate::tests::topic::unique_name();
        let domain = crate::Domain::new(domain_id).unwrap();

        let participant = crate::Participant::new(&domain).unwrap();
        let qos = crate::QoS::new()
            .with_destination_order(crate::qos::policy::DestinationOrder::BySourceTimestamp);
        let topic = crate::Topic::<crate::tests::topic::Data>::builder(&participant, &topic_name)
            .with_qos(&qos)
            .build()
            .unwrap();

        {
            let _writer = crate::Writer::new(&topic).unwrap();
            let _reader = crate::Reader::builder(&topic)
                .with_qos(
                    &crate::QoS::new().with_durability(crate::qos::policy::Durability::Persistent),
                )
                .with_listener(
                    crate::ReaderListener::new().with_requested_incompatible_qos(
                        |_reader, _metadata| {
                            TRIGGERED.lock().unwrap().requested_incompatible_qos += 1;
                        },
                    ),
                )
                .build()
                .unwrap();
        }

        {
            let qos = crate::QoS::new().with_deadline(crate::qos::policy::Deadline {
                period: crate::Duration::from_nanos(1_000_000),
            });
            let reader = crate::Reader::builder(&topic)
                .with_qos(&qos)
                .with_listener(crate::ReaderListener::new().with_requested_deadline_missed(
                    |_reader, _metadata| {
                        TRIGGERED.lock().unwrap().requested_deadline_missed |= true;
                    },
                ))
                .build()
                .unwrap();
            let writer = crate::Writer::builder(&topic)
                .with_qos(&qos)
                .build()
                .unwrap();

            let sample = crate::tests::topic::Data::default();
            writer.write(&sample).unwrap();

            let samples = reader.take().unwrap();
            assert_eq!(samples.len(), 1);
            assert_eq!(*samples[0], sample);

            while !TRIGGERED.lock().unwrap().requested_deadline_missed {
                std::thread::sleep(std::time::Duration::from_nanos(50));
            }
        }

        {
            let reader = crate::Reader::builder(&topic)
                .with_qos(&crate::QoS::new().with_resource_limits(
                    crate::qos::policy::ResourceLimits {
                        max_samples: crate::qos::policy::ResourceLimit::Unlimited,
                        max_instances: crate::qos::policy::ResourceLimit::Limited(1),
                        max_samples_per_instance: crate::qos::policy::ResourceLimit::Unlimited,
                    },
                ))
                .with_listener(crate::ReaderListener::new().with_sample_rejected(
                    |_reader, _metadata| {
                        TRIGGERED.lock().unwrap().sample_rejected += 1;
                    },
                ))
                .build()
                .unwrap();
            let writer = crate::Writer::new(&topic).unwrap();

            let sample = crate::tests::topic::Data {
                x: 1,
                y: 2,
                ..crate::tests::topic::Data::default()
            };
            writer.write(&sample).unwrap();
            writer
                .write(&crate::tests::topic::Data {
                    x: 2,
                    y: 3,
                    ..crate::tests::topic::Data::default()
                })
                .unwrap();

            let samples = reader.take().unwrap();
            assert_eq!(samples.len(), 1);
            assert_eq!(*samples[0], sample);
        }

        {
            let reader = crate::Reader::builder(&topic)
                .with_listener(
                    crate::ReaderListener::new()
                        .with_data_available(|_reader| {
                            TRIGGERED.lock().unwrap().data_available += 1;
                        })
                        .with_subscription_matched(|_reader, _matched| {
                            TRIGGERED.lock().unwrap().subscription_matched += 1;
                        })
                        .with_liveliness_changed(|_reader, _changed| {
                            TRIGGERED.lock().unwrap().liveliness_changed += 1;
                        })
                        .with_sample_lost(|_reader, _metadata| {
                            TRIGGERED.lock().unwrap().sample_lost += 1;
                        }),
                )
                .build()
                .unwrap();
            let writer = crate::Writer::new(&topic).unwrap();

            let sample = crate::tests::topic::Data::default();
            writer.write(&sample).unwrap();

            let key = sample.as_key();
            writer
                .unregister_instance_with_timestamp(
                    &key,
                    (std::time::SystemTime::now() - std::time::Duration::from_secs(1))
                        .try_into()
                        .unwrap(),
                )
                .unwrap();

            let samples = reader.take().unwrap();
            assert_eq!(samples.len(), 1);

            assert_eq!(*samples[0], sample);

            assert_eq!(
                *TRIGGERED.lock().unwrap(),
                Triggered {
                    requested_incompatible_qos: 1,
                    requested_deadline_missed: true,
                    sample_rejected: 1,
                    data_available: 2,
                    subscription_matched: 1,
                    liveliness_changed: 1,
                    sample_lost: 1,
                }
            );
        }
    }

    #[test]
    fn test_writer_listener_callbacks() {
        #[derive(Debug, PartialEq)]
        struct Triggered {
            liveliness_lost: bool,
            offered_deadline_missed: bool,
            offered_incompatible_qos: u32,
            publication_matched: u32,
        }

        static TRIGGERED: std::sync::Mutex<Triggered> = std::sync::Mutex::new(Triggered {
            liveliness_lost: false,
            offered_deadline_missed: false,
            offered_incompatible_qos: 0,
            publication_matched: 0,
        });

        let domain_id = crate::tests::domain::unique_id();
        let topic_name = crate::tests::topic::unique_name();
        let domain = crate::Domain::new(domain_id).unwrap();

        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();

        {
            let _reader = crate::Reader::builder(&topic)
                .with_qos(
                    &crate::QoS::new().with_durability(crate::qos::policy::Durability::Persistent),
                )
                .build()
                .unwrap();
            let _writer = crate::Writer::builder(&topic)
                .with_listener(crate::WriterListener::new().with_offered_incompatible_qos(
                    |_writer, _metadata| {
                        TRIGGERED.lock().unwrap().offered_incompatible_qos += 1;
                    },
                ))
                .build()
                .unwrap();
        }

        {
            let qos = crate::QoS::new().with_deadline(crate::qos::policy::Deadline {
                period: crate::Duration::from_nanos(1_000_000),
            });
            let writer = crate::Writer::builder(&topic)
                .with_qos(&qos)
                .with_listener(crate::WriterListener::new().with_offered_deadline_missed(
                    |_writer, _metadata| {
                        TRIGGERED.lock().unwrap().offered_deadline_missed |= true;
                    },
                ))
                .build()
                .unwrap();
            let reader = crate::Reader::builder(&topic)
                .with_qos(&qos)
                .build()
                .unwrap();

            let sample = crate::tests::topic::Data::default();
            writer.write(&sample).unwrap();

            let samples = reader.take().unwrap();
            assert_eq!(samples.len(), 1);
            assert_eq!(*samples[0], sample);

            while !TRIGGERED.lock().unwrap().offered_deadline_missed {
                std::thread::sleep(std::time::Duration::from_nanos(50));
            }
        }

        {
            let writer = crate::Writer::builder(&topic)
                .with_listener(
                    crate::WriterListener::new()
                        .with_liveliness_lost(|_writer, _metadata| {
                            TRIGGERED.lock().unwrap().liveliness_lost |= true;
                        })
                        .with_publication_matched(|_writer, _metadata| {
                            TRIGGERED.lock().unwrap().publication_matched += 1;
                        }),
                )
                .with_qos(&crate::QoS::new().with_liveliness(
                    crate::qos::policy::Liveliness::ManualByParticipant {
                        lease_duration: crate::Duration::from_nanos(1_000_000),
                    },
                ))
                .build()
                .unwrap();

            let reader = crate::Reader::new(&topic).unwrap();

            let sample = crate::tests::topic::Data::default();
            writer.write(&sample).unwrap();

            let key = sample.as_key();
            writer
                .unregister_instance_with_timestamp(
                    &key,
                    (std::time::SystemTime::now() - std::time::Duration::from_secs(1))
                        .try_into()
                        .unwrap(),
                )
                .unwrap();

            let samples = reader.take().unwrap();
            assert_eq!(samples.len(), 1);

            assert_eq!(*samples[0], sample);

            while !TRIGGERED.lock().unwrap().liveliness_lost {
                std::thread::sleep(std::time::Duration::from_nanos(50));
            }
        }

        assert_eq!(
            *TRIGGERED.lock().unwrap(),
            Triggered {
                liveliness_lost: true,
                offered_deadline_missed: true,
                offered_incompatible_qos: 1,
                publication_matched: 2,
            }
        );
    }
}
