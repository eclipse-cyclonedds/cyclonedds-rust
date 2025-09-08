use crate::Result;
use crate::internal::ffi;
use crate::internal::traits::AsFfi;

/// A domain participant.
///
/// A participant is the entry point for all DDS communication within a
/// [`Domain`](crate::Domain). All other entities, including topics, readers,
/// writers, publishers, and subscribers, are created under a participant and
/// are scoped to its lifetime.
///
/// Use [`Participant::new`] for simple construction or [`Participant::builder`]
/// for [`QoS`](crate::QoS) and [`listener`](crate::listener::Listener)
/// configuration.
#[derive(Debug)]
pub struct Participant<'domain> {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom: std::marker::PhantomData<&'domain crate::Domain>,
}

/// Builder for [`Participant`] (accessible via [`Participant::builder`]).
#[derive(Debug)]
pub struct ParticipantBuilder<'domain, 'qos> {
    domain: &'domain crate::Domain,
    qos: Option<&'qos crate::QoS>,
    listener: Option<crate::Listener>,
}

impl<'d, 'q> ParticipantBuilder<'d, 'q> {
    /// Creates a new [`ParticipantBuilder`] for the given
    /// [`Domain`](crate::Domain).
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Domain;
    /// use cyclonedds::builder::ParticipantBuilder;
    ///
    /// let domain = Domain::default();
    /// let participant_builder = ParticipantBuilder::new(&domain);
    /// ```
    #[must_use]
    pub const fn new(domain: &'d crate::Domain) -> Self {
        Self {
            domain,
            qos: None,
            listener: None,
        }
    }

    /// Sets the [`QoS`](crate::QoS) for this participant builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::builder::ParticipantBuilder;
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    /// # use cyclonedds::Domain;
    /// # let domain = Domain::default();
    ///
    /// let qos = QoS::new().with_reliability(policy::Reliability::Reliable {
    ///     max_blocking_time: Duration::from_millis(100),
    /// });
    /// let participant_builder = ParticipantBuilder::new(&domain).with_qos(&qos);
    /// ```
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }

    /// Sets the [`Listener`](crate::Listener) on this participant builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Listener;
    /// use cyclonedds::builder::ParticipantBuilder;
    /// # use cyclonedds::Domain;
    /// # let domain = Domain::default();
    ///
    /// let participant_builder = ParticipantBuilder::new(&domain).with_listener(Listener::new());
    /// ```
    #[must_use]
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: AsRef<crate::Listener>,
    {
        self.listener = Some(*listener.as_ref());
        self
    }

    /// Builds the [`Participant`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the participant failed to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::builder::ParticipantBuilder;
    /// use cyclonedds::qos::policy::Durability;
    /// use cyclonedds::{Domain, QoS};
    ///
    /// let domain = Domain::default();
    /// let qos = QoS::new().with_durability(Durability::TransientLocal);
    /// let participant = ParticipantBuilder::new(&domain).with_qos(&qos).build()?;
    ///
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn build(self) -> Result<Participant<'d>> {
        // NOTE: using `and_then` to avoid ? branch on the listener for coverage
        // since the C lib currently panics on OOM rather than returning null.
        self.listener
            .map(|listener| listener.as_ffi())
            .transpose()
            .and_then(|listener| {
                Ok(Participant {
                    inner: ffi::dds_create_participant(
                        self.domain.id,
                        self.qos.map(|qos| &qos.inner),
                        listener.as_ref(),
                    )?,
                    phantom: std::marker::PhantomData,
                })
            })
    }
}

impl<'d> Participant<'d> {
    /// Creates a new participant in the given [`Domain`](crate::Domain) with
    /// default [`QoS`](crate::QoS) and no
    /// [`listener`](crate::listener::Listener).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the participant fails to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Domain, Participant};
    ///
    /// let domain = Domain::default();
    /// let participant = Participant::new(&domain)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(domain: &'d crate::Domain) -> Result<Self> {
        Self::builder(domain).build()
    }

    /// Returns a [`ParticipantBuilder`] for constructing a participant with
    /// custom [`QoS`](crate::QoS) or a [`listener`](crate::listener::Listener).
    //
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Domain, Participant};
    ///
    /// let domain = Domain::default();
    /// let participant = Participant::builder(&domain).build()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub const fn builder<'q>(domain: &'d crate::Domain) -> ParticipantBuilder<'d, 'q> {
        ParticipantBuilder::new(domain)
    }

    /// Sets the [`Listener`](crate::Listener) on this participant, replacing
    /// any previously set listener.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the listener fails to set.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Domain, Listener, Participant, listener::SubscriberListener};
    ///
    /// let domain = Domain::default();
    /// let mut participant = Participant::new(&domain)?;
    /// let listener =
    ///     Listener::new().with_subscriber(|s| s.with_data_on_readers(|_| println!("data available")));
    /// participant.set_listener(listener)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set_listener<L>(&mut self, listener: L) -> Result<()>
    where
        L: AsRef<crate::Listener>,
    {
        listener
            .as_ref()
            .as_ffi()
            .and_then(|listener| ffi::dds_set_listener(self.inner, Some(listener.inner)))
    }

    /// Removes the listener from this participant.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the listener fails to unset.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Domain, Participant};
    ///
    /// let domain = Domain::default();
    /// let mut participant = Participant::new(&domain)?;
    /// participant.unset_listener()?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn unset_listener(&mut self) -> Result<()> {
        ffi::dds_set_listener(self.inner, None)?;
        Ok(())
    }

    /// Sets the [`Listener`](crate::Listener) on this participant, consuming
    /// and returning `self`.
    ///
    /// Useful for chaining participant construction with listener
    /// configuration.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the listener fails to set.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Domain, Listener, Participant};
    ///
    /// let domain = Domain::default();
    /// let participant = Participant::new(&domain)?.with_listener(Listener::new())?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn with_listener<L>(mut self, listener: L) -> Result<Self>
    where
        L: AsRef<crate::Listener>,
    {
        self.set_listener(listener).map(|()| self)
    }
}

impl Drop for Participant<'_> {
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
    use crate::Error;

    #[test]
    fn test_participant_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();

        let qos = crate::QoS::new();

        let _ = Participant::new(&domain).unwrap();
        let _ = Participant::builder(&domain)
            .with_qos(&qos)
            .build()
            .unwrap();
        let _ = Participant::new(&domain).unwrap();
        let _ = Participant::builder(&domain)
            .with_qos(&qos)
            .build()
            .unwrap();
    }

    #[test]
    fn test_participant_create_in_impossible_domain() {
        let domain = crate::Domain {
            id: u32::from(u16::MAX),
            inner: 0,
        };

        let result = Participant::new(&domain).unwrap_err();
        assert_eq!(result, Error::NonSpecific);

        let qos = crate::QoS::new();
        let result = Participant::builder(&domain)
            .with_qos(&qos)
            .build()
            .unwrap_err();
        assert_eq!(result, Error::NonSpecific);
    }

    #[test]
    fn test_participant_with_listener() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();

        let listener = crate::Listener::new();

        let _ = Participant::new(&domain)
            .unwrap()
            .with_listener(listener)
            .unwrap();
        let _ = Participant::builder(&domain)
            .with_listener(listener)
            .build()
            .unwrap();

        let mut participant = Participant::new(&domain).unwrap();
        participant.set_listener(listener).unwrap();
        participant.unset_listener().unwrap();
    }

    #[test]
    fn test_participant_with_listener_on_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();

        let listener = crate::Listener::new();

        let mut participant = Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = participant.set_listener(listener).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = participant.unset_listener().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        participant.inner = participant_id;
    }
}
