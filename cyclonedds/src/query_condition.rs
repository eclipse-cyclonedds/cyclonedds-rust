use ffi::Filter;

use crate::internal::ffi;
use crate::state::State;
use crate::{Reader, Result};

/// A filter on a [`Reader`](crate::Reader) that restricts samples by their
/// [`State`](crate::State) and a predicate.
///
/// A `QueryCondition` extends [`ReadCondition`](crate::ReadCondition) with a
/// user-supplied predicate `F` that is applied to each sample's payload. Only
/// samples that both match the state mask and satisfy the predicate are
/// returned by reads and trigger waitset wakeups.
///
/// # Predicate requirements
/// The predicate closure provided to the [`QueryCondition`] must be
/// *zero-sized*.
///
/// This restriction comes from how the callback is integrated with the
/// underlying C API. For ergonomics, the closure is reconstructed inside a
/// wrapper that converts raw C types into Rust types before invoking it. That
/// reconstruction depends only on the closure's type, so it cannot rely on any
/// captured state.
///
/// In practice, this means the callback must be either:
///
///   - a [function item](https://doc.rust-lang.org/reference/types/function-item.html),
///     which is always zero-sized, or
///   - a [closure](https://doc.rust-lang.org/reference/types/closure.html) that
///     does not capture any variables from its environment
///
/// The following example will **fail to compile** because the closure captures
/// `x`, making it non-zero-sized:
///
/// ```compile_fail
/// use cyclonedds as dds;
/// use dds::state;
///
/// # #[derive(
/// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
/// # )]
/// struct Data {
///     x: i32,
/// }
/// fn create_your_reader() -> dds::Reader<'static, 'static, 'static, Data> {
///     unimplemented!()
/// }
/// let reader: dds::Reader<Data> = create_your_reader();
/// let x = 10;
/// let result = dds::QueryCondition::new(
///     &reader,
///     state::sample::Any | state::instance::Any | state::view::Any,
///     // Error: closure captures `x`, so it is not zero-sized.
///     |sample| sample.x < x,
/// )?;
/// # Ok::<(), dds::Error>(())
/// ```
///
/// Instead, use a function item or a non-capturing closure, for example:
///
/// ```ignore
/// |sample| sample.x < 10
/// ```
///
/// The compiler will emit an error similar to:
///
/// ```text
/// error[E0080]: evaluation panicked: the provided callback is not zero-sized
///   = note: closures that capture values from their environment are not zero-sized
///   = help: ensure the callback is either:
///           - a function item, e.g. `fn my_callback() {}`
///           - a closure that does not capture any external state
/// ```
///
/// This is enforced via an internal compile-time assertion
/// `assert!(size_of::<F>() == 0)` but note that the associated compiler output
/// can be quite lengthy.
///
/// <details>
/// <summary>Click to see a sample of the full compiler output</summary>
///
/// ```text
/// error[E0080]: evaluation panicked: the provided callback is not zero-sized
///                 = note: closures that capture values from their environment are not zero-sized
///                 = help: ensure the callback is either:
///                         - a function item, e.g. `fn my_callback() {}`
///                         - a closure that does not capture any external state
/// --> cyclonedds/src/internal/ffi.rs
///  |
///  |       const IS_PROVIDED_CALLBACK_ZERO_SIZED: () = assert!(
///  |  _________________________________________________^
///  | |         size_of::<F>() == 0,
///  | |         "\
///  | | the provided callback is not zero-sized
///    |
///  | | "
///  | | );
///  | |_^ evaluation of `<QueryCondition as Filter>::IS_PROVIDED_CALLBACK_ZERO_SIZED` failed here
///
/// note: erroneous constant encountered
/// --> cyclonedds/src/query_condition.rs:84:17
///  |
///  |         Self::IS_PROVIDED_CALLBACK_ZERO_SIZED;
///  |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///
/// ...
///
/// note: erroneous constant encountered
/// --> cyclonedds/src/internal/ffi.rs
///  |
///  |     Callback::IS_PROVIDED_CALLBACK_ZERO_SIZED;
///  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
/// </details>
pub struct QueryCondition<'domain, 'participant, 'topic, 'reader, T, F>
where
    T: crate::Topicable,
    F: Fn(&T) -> bool,
{
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
    phantom_callback: std::marker::PhantomData<F>,
    phantom: std::marker::PhantomData<&'reader Reader<'topic, 'domain, 'participant, T>>,
}

impl<T, F> std::fmt::Debug for QueryCondition<'_, '_, '_, '_, T, F>
where
    T: crate::Topicable,
    F: Fn(&T) -> bool,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryCondition")
            .field("inner", &self.inner)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<T, F> Filter<T, F> for QueryCondition<'_, '_, '_, '_, T, F>
where
    T: crate::Topicable + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
    F: Fn(&T) -> bool,
{
}

impl<'d, 'p, 't, 'r, T, F> QueryCondition<'d, 'p, 't, 'r, T, F>
where
    T: crate::Topicable + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
    F: Fn(&T) -> bool,
{
    /// Creates a new `QueryCondition` on `reader` that matches samples whose
    /// state satisfies `mask` and whose payload satisfies the predicate `F`.
    ///
    /// The predicate is passed by value but must be zero-sized. Non-capturing
    /// closures and function pointers satisfy this requirement. The
    /// [`UnwindSafe`](std::panic::UnwindSafe) bound is required because the
    /// predicate is called across the FFI boundary from within Cyclone DDS.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if `F` is not zero-sized or if the
    /// query condition fails to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
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
    /// let condition =
    ///     QueryCondition::new(&reader, state::sample::Fresh, |sample: &Data| sample.x > 10)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(reader: &'r Reader<'d, 'p, 't, T>, mask: State, _: F) -> Result<Self> {
        let _ = Self::IS_PROVIDED_CALLBACK_ZERO_SIZED;
        let inner = ffi::dds_create_querycondition::<T, F, Self>(reader.inner, mask.bits())?;
        Ok(Self {
            inner,
            phantom_callback: std::marker::PhantomData,
            phantom: std::marker::PhantomData,
        })
    }

    /// Returns the state mask this condition was created with.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the mask returned by the query
    /// condition is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
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
    /// let condition = QueryCondition::new(&reader, state::sample::Fresh, |_: &Data| true)?;
    /// assert_eq!(condition.mask()?, state::sample::Fresh);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn mask(&self) -> Result<State> {
        let mask = ffi::dds_get_mask(self.inner)?;
        crate::state::State::from_bits(mask).ok_or(crate::error::Error::NonSpecific)
    }

    /// Returns `true` if this condition is currently triggered.
    ///
    /// A condition is triggered when samples matching its mask are available
    /// in the reader cache.
    ///
    /// # Errors
    ///
    /// returns an [`Error`](crate::Error) if the query condition fails to read
    /// the trigger state.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
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
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = QueryCondition::new(
    ///     &reader,
    ///     state::sample::Fresh | state::view::Any | state::instance::Any,
    ///     |sample: &Data| sample.x == 4 && sample.y == 0,
    /// )?;
    /// writer.write(&Data { x: 4, y: 0 })?;
    /// assert!(condition.triggered()?);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn triggered(&self) -> Result<bool> {
        ffi::dds_triggered(self.inner)
    }

    /// Removes and returns all samples matching this condition's mask and
    /// predicate from the reader cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the query condition fails to take
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     #[dds(key)]
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = QueryCondition::new(
    ///     &reader,
    ///     state::sample::Any | state::view::Any | state::instance::Any,
    ///     |sample: &Data| true,
    /// )?;
    ///
    /// writer.write(&Data { x: 1, y: 0 })?;
    /// writer.write(&Data { x: 100, y: 0 })?;
    ///
    /// // Attempt a normal read.
    /// assert_eq!(reader.peek()?.len(), 2);
    ///
    /// // Both samples should match.
    /// let samples = condition.take()?;
    /// assert_eq!(samples.len(), 2);
    ///
    /// // Samples should be removed from the cache.
    /// assert_eq!(condition.take()?.len(), 0);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn take(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_take(self.inner)
    }

    /// Returns all samples matching this condition's mask and predicate without
    /// marking them as read or removing them from the cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the query condition fails to peek
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     #[dds(key)]
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = QueryCondition::new(
    ///     &reader,
    ///     state::sample::Any | state::view::Any | state::instance::Any,
    ///     |sample: &Data| true,
    /// )?;
    ///
    /// writer.write(&Data { x: 1, y: 0 })?;
    /// writer.write(&Data { x: 100, y: 0 })?;
    ///
    /// // Attempt a normal read.
    /// assert_eq!(reader.read()?.len(), 2);
    ///
    /// // Both samples should match.
    /// let samples = condition.read()?;
    /// assert_eq!(samples.len(), 2);
    ///
    /// // Samples remain in the cache.
    /// assert_eq!(condition.read()?.len(), 2);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn read(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_read(self.inner)
    }

    /// Returns all samples matching this condition's mask and predicate without
    /// marking them as read or removing them from the cache.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the query condition fails to peek
    /// samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{QueryCondition, state};
    /// # use cyclonedds::{Domain, Participant, Topic, Reader, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Debug, Default,
    /// # )]
    /// # struct Data {
    /// #     #[dds(key)]
    /// #     x: i32,
    /// #     y: i32,
    /// # }
    ///
    /// let topic = Topic::<Data>::new(&participant, "MyTopic")?;
    /// let reader = Reader::new(&topic)?;
    /// let writer = Writer::new(&topic)?;
    ///
    /// let condition = QueryCondition::new(
    ///     &reader,
    ///     state::sample::Any | state::view::Any | state::instance::Any,
    ///     |sample: &Data| true,
    /// )?;
    ///
    /// writer.write(&Data { x: 1, y: 0 })?;
    /// writer.write(&Data { x: 100, y: 0 })?;
    ///
    /// // Attempt a normal peek.
    /// assert_eq!(reader.peek()?.len(), 2);
    ///
    /// // Both samples should match.
    /// let samples = condition.peek()?;
    /// assert_eq!(samples.len(), 2);
    ///
    /// // Samples remain in the cache.
    /// assert_eq!(condition.peek()?.len(), 2);
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn peek(&self) -> Result<Vec<crate::sample::SampleOrKey<T>>>
    where
        T: std::clone::Clone,
    {
        ffi::dds_peek(self.inner)
    }
}

impl<T, F> Drop for QueryCondition<'_, '_, '_, '_, T, F>
where
    T: crate::Topicable,
    F: Fn(&T) -> bool,
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
    use crate::state;

    fn query(data: &crate::tests::topic::Data) -> bool {
        assert_eq!(data.x, 101);
        true
    }

    #[test]
    fn test_query_condition_create() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let _ = QueryCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
            query,
        )
        .unwrap();
    }

    #[test]
    fn test_query_condition_create_with_invalid_reader() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let mut reader = crate::Reader::new(&topic).unwrap();
        let reader_id = reader.inner;
        reader.inner = 0;
        let result = QueryCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
            query,
        )
        .unwrap_err();
        reader.inner = reader_id;
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_query_condition_debug_formatting() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let query_condition = QueryCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
            query,
        )
        .unwrap();

        let result = format!("{query_condition:?}");
        assert!(result.contains(&format!("{}", query_condition.inner)));
    }

    #[test]
    fn test_query_condition_get_mask() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();

        let mask = state::sample::Any | state::instance::Any | state::view::Any;

        let query_condition = QueryCondition::new(&reader, mask, query).unwrap();
        let result = query_condition.mask().unwrap();
        assert_eq!(result, mask);

        let mask = state::sample::Fresh | state::instance::Unregistered | state::view::Old;
        let result = query_condition.mask().unwrap();
        assert_ne!(result, mask);

        let read_condition = QueryCondition::new(&reader, mask, |_| false).unwrap();
        let result = read_condition.mask().unwrap();
        assert_eq!(result, mask);
    }

    #[test]
    fn test_query_condition_get_mask_on_invalid_query_condition() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let mut query_condition = QueryCondition::new(
            &reader,
            state::sample::Any | state::instance::Any | state::view::Any,
            query,
        )
        .unwrap();
        let query_condition_id = query_condition.inner;
        query_condition.inner = 0;
        let result = query_condition.mask().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        let result = query_condition.triggered().unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
        query_condition.inner = query_condition_id;
    }

    #[test]
    fn test_query_condition_triggering_reads() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();

        let mask = state::sample::Stale | state::instance::Any | state::view::Any;

        let query_condition = QueryCondition::new(&reader, mask, query).unwrap();

        let sample = crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: "hello".to_string(),
        };
        writer.write(&sample).unwrap();

        let query_condition_received = query_condition.read().unwrap();
        assert_eq!(query_condition_received.len(), 0);
        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert_eq!(reader_received.len(), 1);
        assert_eq!(*reader_received[0], sample);
        assert_eq!(
            reader_received[0].info().state,
            state::sample::Fresh | state::view::New | state::instance::Alive
        );

        let triggered = query_condition.triggered().unwrap();
        assert!(triggered);

        let query_condition_received = query_condition.peek().unwrap();
        assert_eq!(query_condition_received.len(), 1);
        assert_eq!(*query_condition_received[0], sample);

        let triggered = query_condition.triggered().unwrap();
        assert!(triggered);

        let query_condition_received = query_condition.take().unwrap();
        assert_eq!(query_condition_received.len(), 1);
        assert_eq!(*query_condition_received[0], sample);

        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert!(reader_received.is_empty());

        let query_condition_received = query_condition.read().unwrap();
        assert!(query_condition_received.is_empty());
    }

    #[test]
    fn test_query_condition_non_triggering_reads() {
        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let reader = crate::Reader::new(&topic).unwrap();
        let writer = crate::Writer::new(&topic).unwrap();

        let mask = state::sample::Stale | state::instance::Any | state::view::Any;

        let query_condition = QueryCondition::new(&reader, mask, |_| false).unwrap();

        let sample = crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: "hello".to_string(),
        };
        writer.write(&sample).unwrap();

        let query_condition_received = query_condition.read().unwrap();
        assert!(query_condition_received.is_empty());
        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert_eq!(reader_received.len(), 1);
        assert_eq!(*reader_received[0], sample);
        assert_eq!(
            reader_received[0].info().state,
            state::sample::Fresh | state::view::New | state::instance::Alive
        );

        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let query_condition_received = query_condition.peek().unwrap();
        assert!(query_condition_received.is_empty());

        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let query_condition_received = query_condition.take().unwrap();
        assert_eq!(query_condition_received.len(), 0);

        let triggered = query_condition.triggered().unwrap();
        assert!(!triggered);

        let reader_received = reader.read().unwrap();
        assert_eq!(reader_received.len(), 1);

        let query_condition_received = query_condition.read().unwrap();
        assert!(query_condition_received.is_empty());
    }
}
