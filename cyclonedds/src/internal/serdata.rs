//! The [`Serdata`] represents the extension point for language-bindings to
//! interact with serialized sample data in Cyclone.
//!
//! A serdata's payload is either owned by the binding or borrowed from memory a
//! PSMX Instance loaned to the reader, which is what makes a zero-copy receive
//! free of copies. Reading through the borrowed form is unchecked by
//! construction, so this module deals in raw pointers.
#![allow(unsafe_code)]

use crate::internal::ffi;
use crate::internal::sertype::Sertype;
use crate::internal::traits::{CdrHeader, Hash32};
use crate::sample::SampleOrKeyInner as SampleOrKey;

/// Where the sample a [`Serdata`] carries actually lives.
#[derive(Debug)]
pub(crate) enum Payload<T>
where
    T: crate::Topicable,
{
    /// Owned by the binding.
    Owned(std::sync::Arc<SampleOrKey<T>>),
    /// A sample in memory a PSMX Instance loaned to the reader, which is what
    /// makes a zero-copy receive free of copies.
    ///
    /// The loan is kept alive by the reference the serdata holds in
    /// [`ddsi_serdata::loan`][cyclonedds_sys::ddsi_serdata::loan], released by
    /// [`serdata_ops::free`][crate::internal::ffi::serdata_ops::free]. Its key is
    /// computed up front rather than on demand, which is both what the C
    /// library's own sertype does and what keeps the serdata free of interior
    /// mutability that Cyclone DDS's threads could race on.
    Loaned {
        sample: *const T,
        kind: Kind,
        key: T::Key,
    },
}

/// The extension point for wrapping [`cyclonedds_sys::ddsi_serdata`].
#[repr(C)]
#[derive(Debug)]
pub struct Serdata<T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::ddsi_serdata,
    pub(crate) serialized_sample: std::cell::OnceCell<Vec<u8>>,
    pub(crate) payload: Payload<T>,
}

impl<T> Serdata<T>
where
    T: crate::Topicable,
{
    /// Create a new [`Serdata`] of a specific kind associated with a
    /// corresponding [`Sertype`].
    pub fn new<R>(sertype: &Sertype<T, R>, sample: SampleOrKey<T>) -> Self {
        let kind = match sample {
            SampleOrKey::Sample { .. } => Kind::Data,
            SampleOrKey::Key { .. } => Kind::Key,
        };

        Self::with_payload(sertype, kind, Payload::Owned(std::sync::Arc::new(sample)))
    }

    /// Create a new [`Serdata`] whose sample stays in the loaned memory a PSMX
    /// Instance delivered it in.
    ///
    /// ## Safety
    /// - `sample` must point to a fully-initialized `T` that stays valid for as long as this
    ///   [`Serdata`] does, which for a PSMX loan means the caller must give the [`Serdata`] a
    ///   reference to that loan.
    /// - A key-only loan still carries a whole sample, whose key fields hold the value.
    pub(crate) unsafe fn new_loaned<R>(
        sertype: &Sertype<T, R>,
        kind: Kind,
        sample: *const T,
    ) -> Self {
        // SAFETY: guaranteed by the caller.
        let key = crate::Topicable::as_key(unsafe { &*sample });

        Self::with_payload(sertype, kind, Payload::Loaned { sample, kind, key })
    }

    fn with_payload<R>(sertype: &Sertype<T, R>, kind: Kind, payload: Payload<T>) -> Self {
        let inner = ffi::ddsi_serdata_new(&sertype.inner, kind.into());

        let mut serdata = Self {
            inner,
            payload,
            serialized_sample: std::cell::OnceCell::new(),
        };
        serdata.inner.hash = serdata.key().hash32();
        serdata
    }

    /// Get a reference to the sample stored by the [`Serdata`].
    pub fn sample(&self) -> &T {
        match &self.payload {
            Payload::Owned(sample) => sample.as_ref().sample(),
            // A loaned payload is a whole sample either way: the raw
            // representation of a key is a sample whose key fields carry it.
            //
            // SAFETY: the loan the pointer addresses is kept alive by the
            // reference this serdata holds.
            Payload::Loaned { sample, .. } => unsafe { &**sample },
        }
    }

    pub fn key(&self) -> &T::Key {
        match &self.payload {
            Payload::Owned(sample) => sample.as_ref().key(),
            Payload::Loaned { key, .. } => key,
        }
    }

    /// A pointer to the sample this [`Serdata`] holds, or [`None`] when it holds
    /// only a key and one would have to be materialized.
    ///
    /// Unlike [`sample`][Self::sample] this never materializes anything, so a
    /// caller holding the serdata across threads cannot race another on the
    /// materialization. The pointer stays valid for as long as the [`Serdata`]
    /// does.
    pub(crate) fn sample_ptr(&self) -> Option<*const T> {
        match &self.payload {
            Payload::Owned(payload) => match payload.as_ref() {
                SampleOrKey::Sample { sample, .. } => Some(std::ptr::from_ref(sample.as_ref())),
                SampleOrKey::Key { .. } => None,
            },
            Payload::Loaned { sample, .. } => Some(*sample),
        }
    }

    /// CDR-encode the payload, encapsulation header included, into `buffer`.
    fn encode(&self, buffer: &mut Vec<u8>) -> Result<(), cdr_encoding::Error> {
        buffer.extend(&byteorder::NativeEndian::cdr_header());

        match self.kind() {
            Kind::Data => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(buffer, self.sample())
            }
            Kind::Key => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(buffer, self.key())
            }
        }
    }

    /// The CDR form of this serdata's payload, encoded on first use.
    pub fn serialized(&self) -> Result<&[u8], cdr_encoding::Error> {
        // NOTE: this initially used self.serialized_sample.get_or_init() but since the
        // serialization can fail this has to be done as follows.
        //
        // TODO: swap this out for `get_or_try_init()` when that is available.
        if self.serialized_sample.get().is_none() {
            let mut buffer = Vec::new();
            self.encode(&mut buffer)?;
            // Cannot fail: nothing has been set in this branch.
            let _ = self.serialized_sample.set(buffer);
        }

        Ok(self
            .serialized_sample
            .get()
            .expect("the serialized sample was just set"))
    }

    /// As [`serialized`][Self::serialized], but padded out to at least `size`
    /// bytes.
    ///
    /// Cyclone DDS asks for a specific length when it hands out a
    /// serialized-data reference, and expects to get exactly that much.
    pub fn serialized_with_size_hint(&mut self, size: usize) -> Result<&[u8], cdr_encoding::Error> {
        if let Some(serialized_sample) = self.serialized_sample.get_mut() {
            serialized_sample.resize(size.max(serialized_sample.len()), 0);
        } else {
            let mut buffer = Vec::with_capacity(size);
            self.encode(&mut buffer)?;
            buffer.resize(size.max(buffer.len()), 0);
            // Cannot fail: nothing has been set in this branch.
            let _ = self.serialized_sample.set(buffer);
        }

        Ok(self
            .serialized_sample
            .get()
            .expect("the serialized sample was just set or resized"))
    }

    /// Get the kind associated with this serdata.
    pub fn kind(&self) -> Kind {
        match &self.payload {
            Payload::Owned(payload) => match payload.as_ref() {
                SampleOrKey::Sample { .. } => Kind::Data,
                SampleOrKey::Key { .. } => Kind::Key,
            },
            Payload::Loaned { kind, .. } => *kind,
        }
    }
}

/// The possible states for a [`Serdata`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// Contains the serialized key-hash.
    Key,
    /// Contains the serialized data.
    Data,
}

impl TryFrom<cyclonedds_sys::ddsi_serdata_kind> for Kind {
    type Error = crate::Error;

    #[inline]
    fn try_from(
        value: cyclonedds_sys::ddsi_serdata_kind,
    ) -> std::result::Result<Self, Self::Error> {
        match value {
            // Was added in OpenSlice when coherency support was added Because a
            // way to pass through just metadata was needed (perhaps just for
            // group coherency). For example, to send a message that this is the
            // end of a coherent set. This should not be exposed by the C
            // library.
            cyclonedds_sys::ddsi_serdata_kind_SDK_EMPTY => {
                unreachable!("SDK EMPTY should never be exposed by the C library")
            }
            cyclonedds_sys::ddsi_serdata_kind_SDK_KEY => Ok(Kind::Key),
            cyclonedds_sys::ddsi_serdata_kind_SDK_DATA => Ok(Kind::Data),
            _ => Err(crate::Error::BadParameter),
        }
    }
}

impl From<Kind> for cyclonedds_sys::ddsi_serdata_kind {
    #[inline]
    fn from(val: Kind) -> Self {
        match val {
            Kind::Key => cyclonedds_sys::ddsi_serdata_kind_SDK_KEY,
            Kind::Data => cyclonedds_sys::ddsi_serdata_kind_SDK_DATA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serdata_kind_conversion() {
        let actual = cyclonedds_sys::ddsi_serdata_kind_SDK_KEY.try_into();
        let expected = Ok(Kind::Key);
        assert_eq!(actual, expected);

        let actual = cyclonedds_sys::ddsi_serdata_kind_SDK_DATA.try_into();
        let expected = Ok(Kind::Data);
        assert_eq!(actual, expected);

        let actual: Result<Kind, _> = cyclonedds_sys::ddsi_serdata_kind::MAX.try_into();
        let expected = Err(crate::Error::BadParameter);
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic = "internal error: entered unreachable code: SDK EMPTY should never be exposed \
                      by the C library"]
    fn test_serdata_kind_sdk_empty_conversion_panics() {
        let _ = Kind::try_from(cyclonedds_sys::ddsi_serdata_kind_SDK_EMPTY);
    }

    #[test]
    fn test_serdata_request_initial_serialization_with_size() {
        use crate::Topicable;

        let type_name =
            std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
        let topic_has_key = crate::tests::topic::Data::IS_KEYED;
        let mut sertype = Box::new(Sertype::<_, crate::representation::Serialized>::new(
            &type_name,
            topic_has_key,
        ));
        let sample = crate::sample::SampleOrKeyInner::<crate::tests::topic::Data>::new_sample(
            crate::tests::topic::Data::default(),
        );
        let key = crate::sample::SampleOrKeyInner::<crate::tests::topic::Data>::new_key(
            Default::default(),
        );

        let mut sample_serdata = Serdata::new(&sertype, sample);
        let mut key_serdata = Serdata::new(&sertype, key);

        let serialized_01: Vec<_> = sample_serdata
            .serialized_with_size_hint(20)
            .unwrap()
            .to_vec();
        assert_eq!(serialized_01.len(), 20);
        let serialized_02: Vec<_> = sample_serdata
            .serialized_with_size_hint(20)
            .unwrap()
            .to_vec();
        assert_eq!(serialized_02.len(), 20);
        assert_eq!(serialized_01, serialized_02);

        let serialized_01: Vec<_> = key_serdata.serialized_with_size_hint(20).unwrap().to_vec();
        assert_eq!(serialized_01.len(), 20);
        let serialized_02: Vec<_> = key_serdata.serialized_with_size_hint(20).unwrap().to_vec();
        assert_eq!(serialized_02.len(), 20);
        assert_eq!(serialized_01, serialized_02);

        ffi::ddsi_sertype_unref(&mut sertype.inner);
        let _ = Box::into_raw(sertype);
    }
}
