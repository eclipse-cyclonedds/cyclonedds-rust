use crate::internal::ffi;
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
}

impl<'d, 'p, 'q> SubscriberBuilder<'d, 'p, 'q> {
    /// Creates a new [`SubscriberBuilder`] for the given [`Participant`].
    #[must_use]
    pub const fn new(participant: &'p Participant<'d>) -> Self {
        Self {
            participant,
            qos: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this subscriber builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Builds the [`Subscriber`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the subscriber failed to create.
    pub fn build(self) -> Result<Subscriber<'d, 'p>> {
        Ok(Subscriber {
            inner: ffi::dds_create_subscriber(
                self.participant.inner,
                self.qos.map(|qos| &qos.inner),
                None,
            )?,
            phantom: std::marker::PhantomData,
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
}
