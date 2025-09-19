use crate::internal::ffi;
use crate::internal::traits::AsFfi;
use crate::{Participant, Result};

/// A `Publisher` groups [`Writers`](crate::Writer) and controls their shared
/// [`QoS`](crate::QoS). Writers created under a publisher inherit its
/// [`QoS`](crate::QoS) where applicable.
///
/// Use [`Publisher::new`] for simple construction or [`Publisher::builder`] for
/// [`QoS`](crate::QoS) and [`listener`](crate::listener::PublisherListener)
/// configuration.
///
/// In most applications a publisher is created implicitly when constructing a
/// [`Writer`](crate::Writer) directly. Use an explicit publisher when you need
/// coordinated writes across multiple writers.
#[derive(Debug)]
pub struct Publisher<'domain, 'participant> {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom: std::marker::PhantomData<&'participant Participant<'domain>>,
}

/// Builder for [`Publisher`] (accessible via [`Publisher::builder`]).
#[derive(Debug)]
pub struct PublisherBuilder<'domain, 'participant, 'qos> {
    participant: &'participant Participant<'domain>,
    qos: Option<&'qos crate::QoS>,
    listener: Option<crate::PublisherListener>,
}

impl<'d, 'p, 'q> PublisherBuilder<'d, 'p, 'q> {
    /// Creates a new [`PublisherBuilder`] for the given [`Participant`].
    #[must_use]
    pub const fn new(participant: &'p Participant<'d>) -> Self {
        Self {
            participant,
            qos: None,
            listener: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this publisher builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Sets the [`Listener`](crate::Listener) on this publisher builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Listener;
    /// use cyclonedds::builder::PublisherBuilder;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let publisher_builder = PublisherBuilder::new(&participant).with_listener(Listener::new());
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: AsRef<crate::PublisherListener>,
    {
        self.listener = Some(*listener.as_ref());
        self
    }

    /// Builds the [`Publisher`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher failed to create.
    pub fn build(self) -> Result<Publisher<'d, 'p>> {
        // NOTE: using `and_then` to avoid ? branch on the listener for coverage
        // since the C lib currently panics on OOM rather than returning null.
        self.listener
            .map(|listener| listener.as_ffi())
            .transpose()
            .and_then(|listener| {
                Ok(Publisher {
                    inner: ffi::dds_create_publisher(
                        self.participant.inner,
                        self.qos.map(|qos| &qos.inner),
                        listener.as_ref(),
                    )?,
                    phantom: std::marker::PhantomData,
                })
            })
    }
}

impl<'d, 'p> Publisher<'d, 'p> {
    /// Creates a new `Publisher` under `participant` with default
    /// [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::PublisherListener).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher fails to create.
    pub fn new(participant: &'p Participant<'d>) -> Result<Self> {
        Self::builder(participant).build()
    }

    /// Returns a [`PublisherBuilder`](crate::builder::PublisherBuilder) for
    /// constructing a publisher with custom [`QoS`](crate::QoS) or a
    /// [`listener`](crate::listener::PublisherListener).
    #[must_use]
    pub const fn builder<'q>(participant: &'p Participant<'d>) -> PublisherBuilder<'d, 'p, 'q> {
        PublisherBuilder::new(participant)
    }

    /// (WARN: unimplemented in C lib): Suspends publication on all writers
    /// belonging to this publisher.
    ///
    /// <div class="warning">
    ///
    /// This function is currently not implemented by the underlying C library
    /// and will thus always return an unsupported error.
    ///
    /// </div>
    ///
    /// While suspended, calls to [`Writer::write`](crate::Writer::write) may
    /// be batched by the middleware. Call [`resume`](Publisher::resume) to
    /// flush and resume normal publication. Suspend and resume are typically
    /// used together to send a coherent set of updates.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if publisher fails to suspend.
    pub fn suspend(&self) -> Result<()> {
        ffi::dds_suspend(self.inner)
    }

    /// (WARN: unimplemented in C lib): Resumes publication on all writers
    /// belonging to this publisher.
    ///
    /// <div class="warning">
    ///
    /// This function is currently not implemented by the underlying C library
    /// and will thus always return an unsupported error.
    ///
    /// </div>
    ///
    /// Flushes any writes that were batched during a
    /// [`suspend`](Publisher::suspend) and resumes normal publication.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher fails to resume.
    pub fn resume(&self) -> Result<()> {
        ffi::dds_resume(self.inner)
    }

    /// (WARN: unimplemented in C lib): Blocks until all samples written by
    /// writers under this publisher have been acknowledged by all matched
    /// reliable readers, or until `timeout` elapses.
    ///
    /// <div class="warning">
    ///
    /// This function is currently not implemented by the underlying C library
    /// and will thus always return an unsupported error.
    ///
    /// </div>
    ///
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the timeout elapses before all
    /// acknowledgements are received or if the publisher returns an error.
    pub fn wait_for_acks(&self, timeout: crate::Duration) -> Result<()> {
        ffi::dds_wait_for_acks(self.inner, timeout.inner)
    }

    #[allow(unused)]
    pub(crate) const fn from_existing(
        inner: cyclonedds_sys::dds_entity_t,
    ) -> std::mem::ManuallyDrop<Self> {
        std::mem::ManuallyDrop::new(Self {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    /// Sets the [`PublisherListener`](crate::PublisherListener) on this
    /// publisher, replacing any previously set listener.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::PublisherListener;
    /// # use cyclonedds::{Domain, Participant, Publisher};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let mut publisher = Publisher::new(&participant)?;
    /// publisher.set_listener(PublisherListener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set_listener<L>(&mut self, listener: L) -> Result<()>
    where
        L: AsRef<crate::PublisherListener>,
    {
        listener
            .as_ref()
            .as_ffi()
            .and_then(|listener| ffi::dds_set_listener(self.inner, Some(listener.inner)))
    }

    /// Removes the listener from this publisher.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher fails to unset the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant, Publisher};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// let mut publisher = Publisher::new(&participant)?;
    /// publisher.unset_listener()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn unset_listener(&mut self) -> Result<()> {
        ffi::dds_set_listener(self.inner, None)?;
        Ok(())
    }

    /// Sets the [`PublisherListener`](crate::PublisherListener) on this
    /// publisher, consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the publisher fails to set the
    /// listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::PublisherListener;
    /// # use cyclonedds::{Domain, Participant, Publisher};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let publisher = Publisher::new(&participant)?.with_listener(PublisherListener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn with_listener<L>(mut self, listener: L) -> Result<Self>
    where
        L: AsRef<crate::PublisherListener>,
    {
        self.set_listener(listener).map(|()| self)
    }
}

impl Drop for Publisher<'_, '_> {
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
    fn test_publisher_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let participant = Participant::new(&domain).unwrap();
        let _ = Publisher::new(&participant).unwrap();
        let _ = Publisher::builder(&participant)
            .with_qos(&qos)
            .build()
            .unwrap();
    }

    #[test]
    fn test_publisher_create_with_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let qos = crate::QoS::new();
        let mut participant = Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = Publisher::new(&participant).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = Publisher::builder(&participant)
            .with_qos(&qos)
            .build()
            .unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        participant.inner = participant_id;
    }

    #[test]
    fn test_publisher_from_existing_publisher() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let publisher = Publisher::new(&participant).unwrap();

        let new_publisher = Publisher::from_existing(publisher.inner);

        assert_eq!(new_publisher.inner, publisher.inner);
    }

    #[test]
    fn test_publisher_suspend_not_yet_supported_by_c_lib() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let publisher = Publisher::new(&participant).unwrap();

        let result = publisher.suspend();
        assert_eq!(
            result,
            Err(crate::Error::Unsupported),
            "result was not unsupported (might be implemented now?)"
        );
    }

    #[test]
    fn test_publisher_resume_not_yet_supported_by_c_lib() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let publisher = Publisher::new(&participant).unwrap();

        let result = publisher.resume();
        assert_eq!(
            result,
            Err(crate::Error::Unsupported),
            "result was not unsupported (might be implemented now?)"
        );
    }

    #[test]
    fn test_publisher_wait_for_acks_not_yet_supported_by_c_lib() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let publisher = Publisher::new(&participant).unwrap();

        let result =
            publisher.wait_for_acks(std::time::Duration::from_millis(10).try_into().unwrap());
        assert_eq!(
            result,
            Err(crate::Error::Unsupported),
            "result was not unsupported (might be implemented now?)"
        );
    }

    #[test]
    fn test_publisher_with_listener() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::PublisherListener::new();

        let _ = Publisher::new(&participant)
            .unwrap()
            .with_listener(listener)
            .unwrap();
        let _ = Publisher::builder(&participant)
            .with_listener(listener)
            .build()
            .unwrap();

        let mut publisher = Publisher::new(&participant).unwrap();
        publisher.set_listener(listener).unwrap();
        publisher.unset_listener().unwrap();
    }

    #[test]
    fn test_publisher_with_listener_on_invalid_publisher() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let listener = crate::PublisherListener::new();

        let mut publisher = Publisher::new(&participant).unwrap();
        let publisher_id = publisher.inner;
        publisher.inner = 0;
        let result = publisher.set_listener(listener).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = publisher.unset_listener().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        publisher.inner = publisher_id;
    }
}
