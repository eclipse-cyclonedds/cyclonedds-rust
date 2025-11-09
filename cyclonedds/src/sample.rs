//! Types representing received DDS samples and their associated metadata.
//!
//! A received sample is either a full data sample of type
//! [`T`](crate::Topicable) or a key-only sample carrying the instance key
//! [`T::Key`](crate::Topicable::Key).
//!
//! This distinction arises in DDS when an instance is disposed or unregistered,
//! the reader receives a notification carrying only the key rather than a full
//! payload.

use crate::Topicable;

#[derive(Clone, Debug)]
pub(crate) enum SampleOrKeyInner<T>
where
    T: crate::Topicable,
{
    Sample {
        sample: Box<T>,
        materialized_key: std::cell::OnceCell<Box<T::Key>>,
    },
    Key {
        key: Box<T::Key>,
        materialized_sample: std::cell::OnceCell<Box<T>>,
    },
}

impl<T> SampleOrKeyInner<T>
where
    T: crate::Topicable,
{
    pub fn new_sample(sample: T) -> Self {
        Self::Sample {
            sample: Box::new(sample),
            materialized_key: std::cell::OnceCell::new(),
        }
    }

    pub fn new_key(key: T::Key) -> Self {
        Self::Key {
            key: Box::new(key),
            materialized_sample: std::cell::OnceCell::new(),
        }
    }

    pub fn key(&self) -> &T::Key {
        match self {
            Self::Sample {
                sample,
                materialized_key,
            } => materialized_key.get_or_init(|| Box::new(sample.as_key())),
            Self::Key { key, .. } => key,
        }
    }

    pub fn sample(&self) -> &T {
        match self {
            Self::Sample { sample, .. } => sample,
            Self::Key {
                key,
                materialized_sample,
            } => materialized_sample.get_or_init(|| Box::new(T::from_key(key))),
        }
    }
}

/// A received sample, which is either a full payload of type
/// [`T`](crate::Topicable) or a key-only payload carrying
/// [`T::Key`](crate::Topicable::Key).
///
/// Key-only samples are produced when an instance is disposed or unregistered
/// by a writer. [`SampleOrKey`] derefs to `T` in both cases: for key-only
/// samples this materializes a default `T` from the key via
/// [`Topicable::from_key`].
///
/// Use [`view`](SampleOrKey::view) to distinguish between the two cases without
/// triggering materialisation.
pub struct SampleOrKey<T>
where
    T: crate::Topicable,
{
    inner: SampleOrKeyInner<T>,
    pub(crate) info: Info,
}

impl<T> std::clone::Clone for SampleOrKey<T>
where
    T: Topicable + std::clone::Clone,
    T::Key: std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            info: self.info,
        }
    }
}

impl<T> std::fmt::Debug for SampleOrKey<T>
where
    T: Topicable + std::fmt::Debug,
    T::Key: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("SampleOrKey");

        let f = match &self.inner {
            SampleOrKeyInner::Sample { sample, .. } => f.field("sample", sample),
            SampleOrKeyInner::Key { key, .. } => f.field("key", key),
        };

        f.field("info", &self.info).finish()
    }
}

impl<T> SampleOrKey<T>
where
    T: crate::Topicable,
{
    /// Create a new sample or key provided a full sample and sample info.
    pub(crate) fn new_sample(sample: T, info: Info) -> Self {
        let inner = SampleOrKeyInner::new_sample(sample);
        Self { inner, info }
    }

    /// Create a new sample or key provided a key and sample info.
    pub(crate) fn new_key(key: T::Key, info: Info) -> Self {
        let inner = SampleOrKeyInner::new_key(key);
        Self { inner, info }
    }

    /// Returns the metadata associated with this sample.
    pub const fn info(&self) -> &Info {
        &self.info
    }

    /// Returns a reference to the full sample payload, or `None` if this is a
    /// key-only sample.
    pub fn sample(&self) -> Option<&T> {
        match &self.inner {
            SampleOrKeyInner::Sample { sample, .. } => Some(sample),
            SampleOrKeyInner::Key { .. } => None,
        }
    }

    /// Consumes `self` and returns the full sample payload, or `None` if this
    /// is a key-only sample.
    pub fn into_sample(self) -> Option<T> {
        match self.inner {
            SampleOrKeyInner::Sample { sample, .. } => Some(*sample),
            SampleOrKeyInner::Key { .. } => None,
        }
    }

    /// Returns `true` if this is a full sample.
    pub const fn is_sample(&self) -> bool {
        matches!(self.inner, SampleOrKeyInner::Sample { .. })
    }

    /// Returns `true` if this is a full sample and `f` returns `true` for its
    /// payload.
    pub fn is_sample_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match &self.inner {
            SampleOrKeyInner::Sample { sample, .. } => f(sample),
            SampleOrKeyInner::Key { .. } => false,
        }
    }

    /// Returns a reference to the instance key, or `None` if this is a full
    /// sample.
    pub fn key(&self) -> Option<&T::Key> {
        match &self.inner {
            SampleOrKeyInner::Sample { .. } => None,
            SampleOrKeyInner::Key { key, .. } => Some(key),
        }
    }

    /// Consumes `self` and returns the instance key, or `None` if this is a
    /// full sample.
    pub fn into_key(self) -> Option<T::Key> {
        match self.inner {
            SampleOrKeyInner::Sample { .. } => None,
            SampleOrKeyInner::Key { key, .. } => Some(*key),
        }
    }

    /// Returns `true` if this is a key-only sample.
    pub const fn is_key(&self) -> bool {
        matches!(self.inner, SampleOrKeyInner::Key { .. })
    }

    /// Returns `true` if this is a key-only sample and `f` returns `true` for
    /// its key.
    pub fn is_key_and(&self, f: impl FnOnce(&T::Key) -> bool) -> bool {
        match &self.inner {
            SampleOrKeyInner::Sample { .. } => false,
            SampleOrKeyInner::Key { key, .. } => f(key),
        }
    }

    /// Returns a borrowed [`View`] of this sample for pattern matching without
    /// triggering key or sample materialisation.
    pub fn view(&self) -> View<'_, T> {
        match &self.inner {
            SampleOrKeyInner::Sample { sample, .. } => View::Sample(sample.as_ref()),
            SampleOrKeyInner::Key { key, .. } => View::Key(key.as_ref()),
        }
    }
}

impl<T> std::ops::Deref for SampleOrKey<T>
where
    T: crate::Topicable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.sample()
    }
}

/// A borrowed view into a [`SampleOrKey`] for pattern matching.
///
/// Obtained via [`SampleOrKey::view`]. Distinguishes between a full sample and
/// a key-only sample without consuming the [`SampleOrKey`] and without
/// implicitly materializing the other half.
pub enum View<'sample, T>
where
    T: Topicable,
{
    /// A full data sample.
    Sample(&'sample T),
    /// A key-only notification, produced when an instance is disposed or
    /// unregistered.
    Key(&'sample T::Key),
}

impl<T> std::fmt::Debug for View<'_, T>
where
    T: Topicable,
    T::Key: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sample(sample) => f.debug_tuple("Sample").field(sample).finish(),
            Self::Key(key) => f.debug_tuple("Key").field(key).finish(),
        }
    }
}

impl<T> std::cmp::PartialEq for View<'_, T>
where
    T: Topicable + std::cmp::PartialEq,
    T::Key: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (View::Sample(lhs), View::Sample(rhs)) => lhs == rhs,
            (View::Key(lhs), View::Key(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

/// Metadata associated with a received sample.
///
/// Attached to every [`SampleOrKey`] and carries the metadata related to the
/// transmission of the sample.
///
/// <div class="warning">
///
/// The `valid_data` flag from the DDS specification is not present here as it
/// is encoded structurally in the type system via [`SampleOrKey`]. A
/// [`View::Sample`] variant guarantees valid data and a [`View::Key`] variant
/// guarantees the absence of it.
///
/// </div>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Info {
    /// [`sample`](crate::state::sample), [`view`](crate::state::view), and
    /// [`instance`](crate::state::instance) state flags at the time of receipt.
    pub state: crate::State,
    /// Timestamp at which the sample was written by the publisher.
    pub source_timestamp: crate::Time,
    /// Handle identifying the instance this sample belongs to.
    pub instance_handle: crate::entity::InstanceHandle,
    /// Handle identifying the writer that published this sample.
    pub publication_handle: crate::entity::InstanceHandle,
    /// Number of times the instance was disposed before this sample was
    /// received.
    pub disposed_generation_count: u32,
    /// Number of times the instance transitioned to the no-writers state before
    /// this sample was received.
    pub no_writers_generation_count: u32,
    /// Position of this sample relative to other samples for the same instance
    /// in the current read or take call.
    pub sample_rank: u32,
    /// Difference in generation count between this sample and the most recent
    /// sample for the same instance in the current read or take call.
    pub generation_rank: u32,
    /// Difference in generation count between this sample and the most recent
    /// sample for the same instance in the reader's cache.
    pub absolute_generation_rank: u32,
}

impl From<&cyclonedds_sys::dds_sample_info> for Info {
    fn from(sample_info: &cyclonedds_sys::dds_sample_info) -> Self {
        #[allow(clippy::cast_sign_loss, clippy::unnecessary_cast)]
        let state = crate::State::from_bits_truncate(sample_info.sample_state as u32)
            | crate::State::from_bits_truncate(sample_info.view_state as u32)
            | crate::State::from_bits_truncate(sample_info.instance_state as u32);
        let instance_handle = crate::entity::InstanceHandle {
            inner: sample_info.instance_handle,
        };
        let publication_handle = crate::entity::InstanceHandle {
            inner: sample_info.publication_handle,
        };
        let source_timestamp = crate::Time::from_nanos(sample_info.source_timestamp);

        let disposed_generation_count = sample_info.disposed_generation_count;
        let no_writers_generation_count = sample_info.no_writers_generation_count;
        let sample_rank = sample_info.sample_rank;
        let generation_rank = sample_info.generation_rank;
        let absolute_generation_rank = sample_info.absolute_generation_rank;

        Self {
            state,
            source_timestamp,
            instance_handle,
            publication_handle,
            disposed_generation_count,
            no_writers_generation_count,
            sample_rank,
            generation_rank,
            absolute_generation_rank,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_sample_callback(sample: &crate::tests::topic::Data) -> bool {
        sample.x.is_multiple_of(2)
    }

    // NOTE: the general interface expects the key to be passed by ref (even if the key is trivially
    // copyable and small).
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_key_callback(key: &(u32, i32)) -> bool {
        key.0.is_multiple_of(2)
    }

    #[test]
    fn test_sample_or_key_sample_ref() {
        let info = Info {
            state: crate::State::empty(),
            source_timestamp: crate::Time::default(),
            instance_handle: crate::entity::InstanceHandle { inner: 0 },
            publication_handle: crate::entity::InstanceHandle { inner: 0 },
            disposed_generation_count: Default::default(),
            no_writers_generation_count: Default::default(),
            sample_rank: Default::default(),
            generation_rank: Default::default(),
            absolute_generation_rank: Default::default(),
        };
        let data = crate::tests::topic::Data {
            x: 10,
            y: 11,
            message: "sample".to_string(),
        };
        let sample = SampleOrKey::new_sample(data.clone(), info);

        assert!(sample.is_sample());
        assert!(sample.is_sample_and(is_sample_callback));
        assert!(!sample.is_key_and(is_key_callback));
        assert!(!sample.is_key());
        assert_eq!(sample.info(), &info);
        assert_eq!(*sample, data);
        assert_eq!(sample.sample().unwrap(), &data);
        assert_eq!(sample.key(), None);
        assert_eq!(sample.clone().into_sample().unwrap(), data);
        assert_eq!(sample.clone().into_key(), None);
    }

    #[test]
    fn test_sample_or_key_key_ref() {
        let info = Info {
            state: crate::State::empty(),
            source_timestamp: crate::Time::default(),
            instance_handle: crate::entity::InstanceHandle { inner: 0 },
            publication_handle: crate::entity::InstanceHandle { inner: 0 },
            disposed_generation_count: Default::default(),
            no_writers_generation_count: Default::default(),
            sample_rank: Default::default(),
            generation_rank: Default::default(),
            absolute_generation_rank: Default::default(),
        };
        let data = crate::tests::topic::Data {
            x: 10,
            y: 11,
            message: String::new(),
        };
        let key = data.as_key();
        let sample = SampleOrKey::<crate::tests::topic::Data>::new_key(key, info);

        assert!(sample.is_key());
        assert!(sample.is_key_and(is_key_callback));
        assert!(!sample.is_sample_and(is_sample_callback));
        assert!(!sample.is_sample());
        assert_eq!(sample.info(), &info);
        assert_eq!(*sample, data);
        assert_eq!(sample.key().unwrap(), &key);
        assert_eq!(sample.sample(), None);
        assert_eq!(sample.clone().into_key().unwrap(), key);
        assert_eq!(sample.clone().into_sample(), None);
    }

    #[test]
    fn test_sample_or_key_view() {
        let info = Info {
            state: crate::State::empty(),
            source_timestamp: crate::Time::default(),
            instance_handle: crate::entity::InstanceHandle { inner: 0 },
            publication_handle: crate::entity::InstanceHandle { inner: 0 },
            disposed_generation_count: Default::default(),
            no_writers_generation_count: Default::default(),
            sample_rank: Default::default(),
            generation_rank: Default::default(),
            absolute_generation_rank: Default::default(),
        };
        let sample_data = crate::tests::topic::Data {
            x: 10,
            y: 11,
            message: "sample".to_string(),
        };
        let sample_key = sample_data.as_key();

        let sample =
            SampleOrKey::<crate::tests::topic::Data>::new_sample(sample_data.clone(), info);
        let key = SampleOrKey::<crate::tests::topic::Data>::new_key(sample_key, info);

        let sample_display = format!("{sample:?}");
        let key_display = format!("{key:?}");
        assert!(sample_display.contains(&format!("{sample_data:?}")));
        assert!(key_display.contains(&format!("{sample_key:?}")));
        assert!(sample_display.contains(&format!("{sample_data:?}")));
        assert!(sample_display.contains(&format!("{info:?}")));
        assert!(key_display.contains(&format!("{sample_key:?}")));
        assert!(key_display.contains(&format!("{info:?}")));

        let view_sample_display = format!("{:?}", sample.view());
        let view_key_display = format!("{:?}", key.view());
        assert!(view_sample_display.contains(&format!("{sample_data:?}")));
        assert!(view_key_display.contains(&format!("{sample_key:?}")));
        assert!(view_sample_display.contains(&format!("{sample_data:?}")));

        assert!(sample.view() != key.view());
        assert_eq!(sample.view(), View::Sample(&sample_data));
        assert_eq!(key.view(), View::Key(&sample_key));
    }
}
