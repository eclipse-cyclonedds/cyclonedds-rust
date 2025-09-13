use crate::entity::{Entity, EntityId};
use crate::internal::ffi;
use crate::{Participant, Result};

/// An entity for blocking until one or more conditions are met.
///
/// A `WaitSet` collects conditions from attached entities and blocks via
/// [`wait`](WaitSet::wait) or [`wait_until`](WaitSet::wait_until) until at
/// least one of them triggers. Entities are attached with an optional typed
/// blob `A` that is returned alongside the triggered condition, allowing the
/// caller to identify which entity triggered the wakeup.
///
/// Pass `()` as `A` when blobs are not needed.
///
/// # Examples
///
/// ```no_run
/// use cyclonedds::{Duration, WaitSet};
/// # use cyclonedds::{Domain, Participant, Topic, Reader};
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
/// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
/// let reader = Reader::new(&topic)?;
///
/// let mut waitset = WaitSet::<()>::new(&participant)?;
/// waitset.attach(&reader, None)?;
/// waitset.wait(Duration::INFINITE)?;
///
/// let samples = reader.take()?;
/// # Ok::<_, cyclonedds::Error>(())
/// ```
pub struct WaitSet<'domain, 'participant, 'attached, A> {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    attached: std::collections::HashMap<EntityId, &'attached dyn Entity>,
    phantom_blobs: std::marker::PhantomData<&'attached A>,
    phantom: std::marker::PhantomData<&'participant Participant<'domain>>,
}

impl<A> std::fmt::Debug for WaitSet<'_, '_, '_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaitSet")
            .field("inner", &self.inner)
            .field("attached", &self.attached.keys())
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'d, 'p, 'a, A> WaitSet<'d, 'p, 'a, A> {
    /// Creates a new `WaitSet` under a `participant`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the waitset fails to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::WaitSet;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(participant: &'p Participant<'d>) -> Result<Self> {
        let inner = ffi::dds_create_waitset(participant.inner)?;
        Ok(Self {
            inner,
            attached: std::collections::HashMap::new(),
            phantom_blobs: std::marker::PhantomData,
            phantom: std::marker::PhantomData,
        })
    }

    /// Attaches `entity` to this waitset with an optional `blob`.
    ///
    /// When the entity triggers a wakeup, the associated `blob` is returned
    /// by [`wait`](WaitSet::wait). If the entity is already attached, this is
    /// a no-op.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the waitset fails to attach the
    /// entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::WaitSet;
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
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
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// waitset.attach(&reader, None)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn attach(&mut self, entity: &'a dyn Entity, blob: Option<&'a A>) -> Result<()> {
        let id = entity.id();
        if !self.attached.contains_key(&id) {
            ffi::dds_waitset_attach(
                self.inner,
                id.inner,
                blob.map_or(std::ptr::null(), |blob| std::ptr::from_ref(blob)) as isize,
            )?;
            self.attached.insert(id, entity);
        }
        Ok(())
    }

    /// Detaches `entity` from this waitset.
    ///
    /// If the entity is not attached, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the waitset fails to detach the
    /// entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::WaitSet;
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
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
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// waitset.attach(&reader, None)?;
    /// waitset.detach(&reader)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn detach(&mut self, entity: &'a dyn Entity) -> Result<()> {
        let entity = entity.id();
        self.detach_id(entity)
    }

    fn detach_id(&mut self, entity_id: EntityId) -> Result<()> {
        if self.attached.contains_key(&entity_id) {
            ffi::dds_waitset_detach(self.inner, entity_id.inner)?;
            self.attached.remove(&entity_id);
        }

        Ok(())
    }

    /// Sets the trigger state of this waitset directly.
    ///
    /// Setting to `true` causes any current or future [`wait`](WaitSet::wait)
    /// call to return immediately. Setting to `false` resets it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the waitset fails to set the
    /// trigger.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::WaitSet;
    /// # use cyclonedds::{Domain, Participant};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// waitset.set_trigger(true)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn set_trigger(&mut self, trigger: bool) -> Result<()> {
        ffi::dds_waitset_set_trigger(self.inner, trigger)
    }

    /// Blocks until at least one attached condition triggers or `timeout`
    /// elapses.
    ///
    /// Returns the blobs associated with the triggered conditions. Pass
    /// [`Duration::INFINITE`](crate::Duration::INFINITE) to block
    /// indefinitely.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the timeout elapses without any
    /// condition triggering or if the waitset returns an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cyclonedds::{Duration, WaitSet};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    /// # let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// # let reader = Reader::new(&topic)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// waitset.attach(&reader, None)?;
    /// waitset.wait(Duration::from_secs(5))?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn wait(&mut self, timeout: crate::Duration) -> Result<Vec<&'a A>> {
        let (_, attachments) =
            ffi::dds_waitset_wait::<A>(self.inner, self.attached.len(), timeout.inner)?;
        Ok(attachments)
    }

    /// Blocks until at least one attached condition triggers or
    /// `absolute_time` is reached.
    ///
    /// Like [`wait`](WaitSet::wait) but takes an absolute [`Time`](crate::Time)
    /// rather than a relative timeout.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the deadline passes without any
    /// condition triggering or if the waitset returns an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cyclonedds::{Time, WaitSet};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    /// # let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// # let reader = Reader::new(&topic)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// waitset.attach(&reader, None)?;
    /// waitset.wait_until(
    ///     (std::time::SystemTime::now() + std::time::Duration::from_secs(5))
    ///         .try_into()
    ///         .unwrap(),
    /// )?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn wait_until(&mut self, absolute_time: crate::Time) -> Result<Vec<&'a A>> {
        let (_, attachments) =
            ffi::dds_waitset_wait_until::<A>(self.inner, self.attached.len(), absolute_time.inner)?;
        Ok(attachments)
    }

    /// Returns `true` if `entity` is currently attached to this waitset.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::WaitSet;
    /// # use cyclonedds::{Domain, Participant, Topic, Reader};
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
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    ///
    /// let mut waitset = WaitSet::<()>::new(&participant)?;
    /// assert!(!waitset.is_attached(&reader));
    /// waitset.attach(&reader, None)?;
    /// assert!(waitset.is_attached(&reader));
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn is_attached(&self, entity: &'a dyn Entity) -> bool {
        self.attached.contains_key(&entity.id())
    }
}

impl<A> Drop for WaitSet<'_, '_, '_, A> {
    fn drop(&mut self) {
        for entity_id in self.attached.keys() {
            let result = ffi::dds_waitset_detach(self.inner, entity_id.inner);
            debug_assert!(
                result.is_ok(),
                "unable to detach entity: {entity_id:?} from {self:?}: {result:?}"
            );
        }

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
    use crate::state;

    #[test]
    fn test_waitset_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let _ = WaitSet::<()>::new(&participant).unwrap();
    }

    #[test]
    fn test_waitset_create_with_invalid_participant() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let mut participant = crate::Participant::new(&domain).unwrap();
        let participant_id = participant.inner;
        participant.inner = 0;
        let result = WaitSet::<()>::new(&participant).unwrap_err();
        participant.inner = participant_id;

        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_waitset_debug_formatting() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let waitset = WaitSet::<()>::new(&participant).unwrap();

        let result = format!("{waitset:?}");
        assert!(result.contains(&format!("{}", waitset.inner)));
    }

    #[test]
    fn test_waitset_attachment() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let mask = state::sample::Any | state::view::Any | state::instance::Any;
        let read_condition = crate::ReadCondition::new(&reader, mask).unwrap();

        let mut waitset = WaitSet::<()>::new(&participant).unwrap();

        let result = waitset.attach(&topic, None);
        assert!(result.is_ok());
        let result = waitset.attach(&topic, None);
        assert!(result.is_ok());
        let result = waitset.attach(&read_condition, None);
        assert!(result.is_ok());

        assert!(waitset.is_attached(&topic));
        assert!(waitset.is_attached(&read_condition));

        let result = waitset.detach(&read_condition);
        assert!(result.is_ok());

        assert!(waitset.is_attached(&topic));
        assert!(!waitset.is_attached(&read_condition));

        let result = waitset.detach(&read_condition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_waitset_attachment_with_invalid_waitset() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let mask = state::sample::Any | state::view::Any | state::instance::Any;
        let read_condition = crate::ReadCondition::new(&reader, mask).unwrap();

        let mut waitset = WaitSet::<()>::new(&participant).unwrap();

        let result = waitset.attach(&topic, None);
        assert!(result.is_ok());

        let waitset_id = waitset.inner;
        waitset.inner = 0;

        let result = waitset.attach(&read_condition, None).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let result = waitset.detach(&topic).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        waitset.inner = waitset_id;
    }

    #[test]
    fn test_waitset_wait() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();
        let mask = state::sample::Any | state::view::Any | state::instance::Any;
        let read_condition_1 = crate::ReadCondition::new(&reader, mask).unwrap();

        let mask = state::sample::Any | state::view::Any | state::instance::Any;
        let read_condition_2 = crate::ReadCondition::new(&reader, mask).unwrap();

        let mask = state::sample::Any | state::view::Any | state::instance::Any;
        let read_condition_3 = crate::ReadCondition::new(&reader, mask).unwrap();

        let attach01 = String::from("hello");
        let attach02 = String::from("world");
        let mut waitset = WaitSet::new(&participant).unwrap();

        waitset.attach(&read_condition_1, Some(&attach01)).unwrap();

        let actual = waitset
            .wait(crate::Duration::from_nanos(5_000_000))
            .unwrap_err();
        assert_eq!(actual, crate::Error::Timeout);

        let actual = waitset.wait_until(crate::Time::from_nanos(0)).unwrap_err();
        assert_eq!(actual, crate::Error::Timeout);

        writer.write(&crate::tests::topic::Data::default()).unwrap();
        let actual = waitset
            .wait(crate::Duration::from_nanos(1_000_000_000))
            .unwrap();
        assert_eq!(actual, vec![&attach01]);

        let actual = waitset.wait_until(crate::Time::from_nanos(1)).unwrap();
        assert_eq!(actual, vec![&attach01]);

        waitset.attach(&read_condition_2, Some(&attach02)).unwrap();
        let actual = waitset
            .wait(crate::Duration::from_nanos(1_000_000_000))
            .unwrap();
        assert_eq!(actual, vec![&attach01, &attach02]);

        let actual = waitset.wait_until(crate::Time::from_nanos(1)).unwrap();
        assert_eq!(actual, vec![&attach01, &attach02]);

        waitset.attach(&read_condition_3, None).unwrap();
        let actual = waitset
            .wait(crate::Duration::from_nanos(1_000_000_000))
            .unwrap();
        assert_eq!(actual, vec![&attach01, &attach02]);

        let actual = waitset.wait_until(crate::Time::from_nanos(1)).unwrap();
        assert_eq!(actual, vec![&attach01, &attach02]);
    }

    #[test]
    fn test_waitset_wait_with_invalid_waitset() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();

        let attach01 = String::from("hello");
        let mut waitset = WaitSet::new(&participant).unwrap();
        waitset.attach(&participant, Some(&attach01)).unwrap();

        let waitset_id = waitset.inner;
        waitset.inner = 0;
        let result = waitset.wait(crate::Duration::INFINITE).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = waitset.wait_until(crate::Time::NEVER).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        waitset.inner = waitset_id;
    }

    #[test]
    fn test_waitset_set_trigger() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut waitset = WaitSet::<()>::new(&participant).unwrap();

        let result = waitset.set_trigger(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_waitset_set_trigger_with_invalid_waitset() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let mut waitset = WaitSet::<()>::new(&participant).unwrap();
        let waitset_id = waitset.inner;
        waitset.inner = 0;

        let result = waitset.set_trigger(true).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        waitset.inner = waitset_id;
    }

    #[test]
    fn test_waitset_wait_dynamic_data() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();

        let data01 = 10;
        let data02 = "String";
        let attach01 = Box::new(data01) as _;
        let attach02 = Box::new(data02) as _;
        let mut waitset = WaitSet::<Box<dyn std::any::Any>>::new(&participant).unwrap();
        waitset.attach(&reader, Some(&attach01)).unwrap();
        waitset.attach(&writer, Some(&attach02)).unwrap();

        writer.write(&crate::tests::topic::Data::default()).unwrap();

        let attachments = waitset.wait(crate::Duration::INFINITE).unwrap();

        assert_eq!(attachments.len(), 2);

        let attach01_result = attachments[0].downcast_ref::<i32>().unwrap();
        let attach02_result = attachments[1].downcast_ref::<&str>().unwrap();

        assert_eq!(*attach01_result, data01);
        assert_eq!(*attach02_result, data02);
    }
}
