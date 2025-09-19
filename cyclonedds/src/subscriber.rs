use crate::internal::ffi;
use crate::internal::traits::AsFfi;
use crate::{Participant, Result};

/// A `Subscriber` groups [`Readers`](crate::Reader) and controls their shared
/// [`QoS`](crate::QoS). Readers created under a subscriber inherit its
/// [`QoS`](crate::QoS) where applicable.
///
/// Use [`Subscriber::new`] for simple construction or [`Subscriber::builder`]
/// for [`QoS`](crate::QoS) and
/// [`listener`](crate::listener::SubscriberListener) configuration.
///
/// In most applications a subscriber is created implicitly when constructing a
/// [`Reader`](crate::Reader) directly. Use an explicit subscriber when you need
/// coordinated reads across multiple readers.
#[derive(Debug)]
pub struct Subscriber<'domain, 'participant> {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom: std::marker::PhantomData<&'participant Participant<'domain>>,
}

/// Builder for [`Subscriber`] (accessible via [`Subscriber::builder`]).
#[derive(Debug)]
pub struct SubscriberBuilder<'domain, 'participant, 'qos> {
    participant: &'participant Participant<'domain>,
    qos: Option<&'qos crate::QoS>,
    listener: Option<crate::SubscriberListener>,
}

impl<'d, 'p, 'q> SubscriberBuilder<'d, 'p, 'q> {
    /// Creates a new [`SubscriberBuilder`] for the given [`Participant`].
    #[must_use]
    pub const fn new(participant: &'p Participant<'d>) -> Self {
        Self {
            participant,
            qos: None,
            listener: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this subscriber builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    ///
    /// Sets the [`Listener`](crate::Listener) on this subscriber builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Listener;
    /// use cyclonedds::builder::SubscriberBuilder;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let subscriber_builder = SubscriberBuilder::new(&participant).with_listener(Listener::new());
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: AsRef<crate::SubscriberListener>,
    {
        self.listener = Some(*listener.as_ref());
        self
    }

    /// Builds the [`Subscriber`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber failed to create.
    pub fn build(self) -> Result<Subscriber<'d, 'p>> {
        // NOTE: using `and_then` to avoid ? branch on the listener for coverage
        // since the C lib currently panics on OOM rather than returning null.
        self.listener
            .map(|listener| listener.as_ffi())
            .transpose()
            .and_then(|listener| {
                Ok(Subscriber {
                    inner: ffi::dds_create_subscriber(
                        self.participant.inner,
                        self.qos.map(|qos| &qos.inner),
                        listener.as_ref(),
                    )?,
                    phantom: std::marker::PhantomData,
                })
            })
    }
}

impl<'d, 'p> Subscriber<'d, 'p> {
    /// Creates a new `Subscriber` under `participant` with default
    /// [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::SubscriberListener).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber fails to create.
    pub fn new(participant: &'p Participant<'d>) -> Result<Self> {
        Self::builder(participant).build()
    }

    /// Returns a [`SubscriberBuilder`](crate::builder::SubscriberBuilder) for
    /// constructing a subscriber with custom [`QoS`](crate::QoS) or a
    /// [`listener`](crate::listener::SubscriberListener).
    #[must_use]
    pub const fn builder<'q>(participant: &'p Participant<'d>) -> SubscriberBuilder<'d, 'p, 'q> {
        SubscriberBuilder::new(participant)
    }

    /// (WARN: unimplemented in C lib): Notifies all readers belonging to this
    /// subscriber that data is available.
    ///
    /// <div class="warning">
    ///
    /// This function is currently not implemented by the underlying C library
    /// and will thus always return an unsupported error.
    ///
    /// </div>
    ///
    /// Triggers the
    /// [`DataOnReaders`](crate::listener::SubscriberListener::with_data_on_readers)
    /// callback on the subscriber's listener and the
    /// [`DataAvailable`](crate::listener::ReaderListener::with_data_available)
    /// callback on each reader's listener.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber fails to notify the
    /// readers.
    pub fn notify_readers(&self) -> Result<()> {
        ffi::dds_notify_readers(self.inner)
    }

    pub(crate) const fn from_existing(
        inner: cyclonedds_sys::dds_entity_t,
    ) -> std::mem::ManuallyDrop<Self> {
        std::mem::ManuallyDrop::new(Self {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    /// Sets the [`SubscriberListener`](crate::SubscriberListener) on this
    /// subscriber, replacing any previously set listener.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::SubscriberListener;
    /// # use cyclonedds::{Domain, Participant, Subscriber};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let mut subscriber = Subscriber::new(&participant)?;
    /// subscriber.set_listener(SubscriberListener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set_listener<L>(&mut self, listener: L) -> Result<()>
    where
        L: AsRef<crate::SubscriberListener>,
    {
        listener
            .as_ref()
            .as_ffi()
            .and_then(|listener| ffi::dds_set_listener(self.inner, Some(listener.inner)))
    }

    /// Removes the listener from this subscriber.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber fails to unset the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant, Subscriber};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// let mut subscriber = Subscriber::new(&participant)?;
    /// subscriber.unset_listener()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn unset_listener(&mut self) -> Result<()> {
        ffi::dds_set_listener(self.inner, None)?;
        Ok(())
    }

    /// Sets the [`SubscriberListener`](crate::SubscriberListener) on this
    /// subscriber, consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::SubscriberListener;
    /// # use cyclonedds::{Domain, Participant, Subscriber};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let subscriber = Subscriber::new(&participant)?.with_listener(SubscriberListener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn with_listener<L>(mut self, listener: L) -> Result<Self>
    where
        L: AsRef<crate::SubscriberListener>,
    {
        self.set_listener(listener).map(|_err| self)
    }
}

impl Drop for Subscriber<'_, '_> {
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
    fn test_subscriber_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let participant = Participant::new(&domain).unwrap();
        let _ = Subscriber::new(&participant).unwrap();
        let _ = Subscriber::builder(&participant)
            .with_qos(&qos)
            .build()
            .unwrap();
    }

    #[test]
    fn test_subscriber_create_with_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let mut participant = Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = Subscriber::new(&participant).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Subscriber::builder(&participant)
            .with_qos(&qos)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        participant.inner = participant_id;
    }

    #[test]
    fn test_subscriber_from_existing_subscriber() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let subscriber = Subscriber::new(&participant).unwrap();

        let new_subscriber = Subscriber::from_existing(subscriber.inner);

        assert_eq!(new_subscriber.inner, subscriber.inner);
    }

    #[test]
    fn test_subscriber_notify_readers_not_yet_supported_by_c_lib() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let subscriber = Subscriber::new(&participant).unwrap();

        let result = subscriber.notify_readers();
        assert_eq!(
            result,
            Err(crate::Error::Unsupported),
            "result was not unsupported (might be implemented now?)"
        );
    }

    #[test]
    fn test_subscriber_with_listener() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::SubscriberListener::new().with_data_on_readers(|_| ());

        let _ = Subscriber::new(&participant)
            .unwrap()
            .with_listener(listener)
            .unwrap();

        let _ = Subscriber::builder(&participant)
            .with_listener(listener)
            .build()
            .unwrap();

        let mut subscriber = Subscriber::new(&participant).unwrap();
        subscriber.set_listener(listener).unwrap();
        subscriber.unset_listener().unwrap();
    }

    #[test]
    fn test_subscriber_with_listener_on_invalid_subscriber() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::SubscriberListener::new().with_data_on_readers(|_| ());

        let mut subscriber = Subscriber::new(&participant).unwrap();
        let subscriber_id = subscriber.inner;
        subscriber.inner = 0;
        let result = subscriber.set_listener(listener).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = subscriber.unset_listener().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        subscriber.inner = subscriber_id;
    }
}
