use crate::Result;
use crate::entity::Entity;
use crate::internal::ffi;

/// A manually triggered condition for use with a [`WaitSet`](crate::WaitSet).
///
/// A `GuardCondition` allows application code to unblock a waiting
/// [`WaitSet`](crate::WaitSet) from another thread or in response to an
/// external event. Set it to `true` to trigger attached waitsets and `false`
/// to reset it.
#[derive(Debug)]
pub struct GuardCondition<'owner> {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom: std::marker::PhantomData<&'owner dyn Entity>,
}

impl<'o> GuardCondition<'o> {
    /// Creates a new `GuardCondition` owned by `owner`.
    ///
    /// The guard condition is valid for the lifetime of `owner`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the guard condition fails to
    /// create.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// use cyclonedds::GuardCondition;
    ///
    /// let guard_condition = GuardCondition::new(&participant)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(owner: &'o dyn Entity) -> Result<Self> {
        let owner = owner.handle().inner;
        let inner = ffi::dds_create_guardcondition(owner)?;
        Ok(Self {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    /// Sets the triggered state of this guard condition.
    ///
    /// Setting to `true` unblocks any [`WaitSet`](crate::WaitSet) this
    /// condition is attached to. Setting to `false` resets it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the condition failed to set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// use cyclonedds::GuardCondition;
    ///
    /// let mut guard_condition = GuardCondition::new(&participant)?;
    /// guard_condition.set(true)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set(&mut self, triggered: bool) -> Result<()> {
        ffi::dds_set_guardcondition(self.inner, triggered)
    }

    /// Returns the current triggered state without clearing it.
    ///
    /// Equivalent to [`read`](GuardCondition::read).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the condition failed to read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// use cyclonedds::GuardCondition;
    ///
    /// let mut guard = GuardCondition::new(&participant)?;
    /// guard.set(true)?;
    /// assert_eq!(guard.peek()?, true);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn peek(&self) -> Result<bool> {
        self.read()
    }

    /// Returns the current triggered state without clearing it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the condition failed to read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// use cyclonedds::GuardCondition;
    ///
    /// let mut guard = GuardCondition::new(&participant)?;
    /// guard.set(true)?;
    /// assert_eq!(guard.read()?, true);
    /// // State is preserved after read.
    /// assert_eq!(guard.read()?, true);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn read(&self) -> Result<bool> {
        ffi::dds_read_guardcondition(self.inner)
    }

    /// Returns the current triggered state and clears it.
    ///
    /// Unlike [`read`](GuardCondition::read), this resets the triggered state
    /// to `false` after returning it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the condition failed to take.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// use cyclonedds::GuardCondition;
    ///
    /// let mut guard = GuardCondition::new(&participant)?;
    /// guard.set(true)?;
    /// assert_eq!(guard.take()?, true);
    /// // State has been cleared.
    /// assert_eq!(guard.read()?, false);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn take(&mut self) -> Result<bool> {
        ffi::dds_take_guardcondition(self.inner)
    }
}

impl Drop for GuardCondition<'_> {
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
    fn test_guard_condition_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let _ = GuardCondition::new(&participant).unwrap();
    }

    #[test]
    fn test_guard_condition_with_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let mut participant = crate::Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = GuardCondition::new(&participant).unwrap_err();
        participant.inner = participant_id;

        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_guard_condition_set() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut guard_condition = GuardCondition::new(&participant).unwrap();

        guard_condition.set(true).unwrap();
        guard_condition.set(false).unwrap();
    }

    #[test]
    fn test_guard_condition_peek() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut guard_condition = GuardCondition::new(&participant).unwrap();

        guard_condition.set(true).unwrap();
        let triggered = guard_condition.peek().unwrap();
        assert!(triggered);
        let triggered = guard_condition.peek().unwrap();
        assert!(triggered);

        guard_condition.set(false).unwrap();
        let triggered = guard_condition.peek().unwrap();
        assert!(!triggered);
        let triggered = guard_condition.peek().unwrap();
        assert!(!triggered);
    }

    #[test]
    fn test_guard_condition_read() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut guard_condition = GuardCondition::new(&participant).unwrap();

        guard_condition.set(true).unwrap();
        let triggered = guard_condition.read().unwrap();
        assert!(triggered);
        let triggered = guard_condition.read().unwrap();
        assert!(triggered);

        guard_condition.set(false).unwrap();
        let triggered = guard_condition.read().unwrap();
        assert!(!triggered);
        let triggered = guard_condition.read().unwrap();
        assert!(!triggered);
    }

    #[test]
    fn test_guard_condition_take() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut guard_condition = GuardCondition::new(&participant).unwrap();

        guard_condition.set(true).unwrap();
        let triggered = guard_condition.take().unwrap();
        assert!(triggered);
        let triggered = guard_condition.take().unwrap();
        assert!(!triggered);

        guard_condition.set(false).unwrap();
        let triggered = guard_condition.take().unwrap();
        assert!(!triggered);
        let triggered = guard_condition.take().unwrap();
        assert!(!triggered);
    }

    #[test]
    fn test_guard_condition_operations_on_invalid_guard_condition() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut guard_condition = GuardCondition::new(&participant).unwrap();
        let guard_condition_id = guard_condition.inner;
        guard_condition.inner = 0;

        let result = guard_condition.set(false).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = guard_condition.read().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = guard_condition.take().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        guard_condition.inner = guard_condition_id;
    }
}
