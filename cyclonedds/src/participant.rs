use crate::Result;
use crate::internal::ffi;

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
}

impl<'d, 'q> ParticipantBuilder<'d, 'q> {
    /// Creates a new [`ParticipantBuilder`] for the given
    /// [`Domain`](crate::Domain).
    #[must_use]
    pub const fn new(domain: &'d crate::Domain) -> Self {
        Self { domain, qos: None }
    }

    /// Sets the [`QoS`](crate::QoS) for this participant builder.
    #[must_use]
    pub const fn with_qos(mut self, qos: &'q crate::QoS) -> Self {
        self.qos = Some(qos);
        self
    }
    /// Builds the [`Participant`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the participant failed to create.
    pub fn build(self) -> Result<Participant<'d>> {
        Ok(Participant {
            inner: ffi::dds_create_participant(
                self.domain.id,
                self.qos.map(|qos| &qos.inner),
                None,
            )?,
            phantom: std::marker::PhantomData,
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
}
