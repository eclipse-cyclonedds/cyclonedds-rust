//! The [`Serdata`] represents the extension point for language-bindings to
//! interact with serialized sample data in Cyclone.

use crate::internal::ffi;
use crate::internal::sertype::Sertype;
use crate::internal::traits::{CdrHeader, Hash32};
use crate::sample::SampleOrKeyInner as SampleOrKey;

/// The extension point for wrapping [`cyclonedds_sys::ddsi_serdata`].
#[repr(C)]
#[derive(Debug)]
pub struct Serdata<T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::ddsi_serdata,
    pub(crate) serialized_sample: std::cell::OnceCell<Vec<u8>>,
    pub(crate) sample: std::sync::Arc<SampleOrKey<T>>,
}

impl<T> Serdata<T>
where
    T: crate::Topicable,
{
    /// Create a new [`Serdata`] of a specific kind associated with a
    /// corresponding [`Sertype`].
    pub fn new(sertype: &Sertype<T>, sample: SampleOrKey<T>) -> Self {
        let kind = match sample {
            SampleOrKey::Sample { .. } => Kind::Data,
            SampleOrKey::Key { .. } => Kind::Key,
        };

        let mut inner = ffi::ddsi_serdata_new(&sertype.inner, kind.into());
        inner.hash = sample.key().hash32();

        Self {
            inner,
            sample: std::sync::Arc::new(sample),
            serialized_sample: std::cell::OnceCell::new(),
        }
    }

    /// Get a reference to the sample stored by the [`Serdata`].
    pub fn sample(&mut self) -> &T {
        self.sample.as_ref().sample()
    }

    pub fn key(&mut self) -> &T::Key {
        self.sample.as_ref().key()
    }

    pub fn serialized(&mut self) -> Result<&[u8], cdr_encoding::Error> {
        // NOTE: this initially used self.serialized_sample.get_or_init() but since the
        // serialization can fail this has to be done as follows.
        //
        // TODO: swap this out for `get_or_try_init()` when that is available.
        if let Some(serialized_sample) = self.serialized_sample.get() {
            Ok(serialized_sample)
        } else {
            let mut serialized: Vec<_> =
                byteorder::NativeEndian::cdr_header().into_iter().collect();
            match self.sample.as_ref() {
                SampleOrKey::Sample { sample, .. } => cdr_encoding::to_writer::<
                    _,
                    byteorder::NativeEndian,
                    _,
                >(&mut serialized, sample),
                SampleOrKey::Key { key, .. } => {
                    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut serialized, key)
                }
            }
            .map(|()| {
                // SAFETY: guaranteed because in this branch the serialized sample is unset.
                self.serialized_sample.set(serialized).unwrap();
                // SAFETY: guaranteed because the set call was successful by here.
                self.serialized_sample.get().unwrap().as_ref()
            })
        }
    }

    pub fn serialized_with_size_hint(&mut self, size: usize) -> Result<&[u8], cdr_encoding::Error> {
        if let Some(serialized_sample) = self.serialized_sample.get_mut() {
            serialized_sample.resize(size.max(serialized_sample.len()), 0);
        }

        // NOTE: this initially used self.serialized_sample.get_or_init() but since the
        // serialization can fail this has to be done as follows.
        //
        // TODO: swap this out for `get_or_try_init()` when that is available.
        if let Some(serialized_sample) = self.serialized_sample.get() {
            Ok(serialized_sample)
        } else {
            let mut serialized = Vec::with_capacity(size);
            serialized.extend(&byteorder::NativeEndian::cdr_header());
            match self.sample.as_ref() {
                SampleOrKey::Sample { sample, .. } => cdr_encoding::to_writer::<
                    _,
                    byteorder::NativeEndian,
                    _,
                >(&mut serialized, sample),
                SampleOrKey::Key { key, .. } => {
                    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut serialized, key)
                }
            }
            .map(|()| {
                serialized.resize(size.max(serialized.len()), 0);
                // SAFETY: guaranteed because in this branch the serialized sample is unset.
                self.serialized_sample.set(serialized).unwrap();
                // SAFETY: guaranteed because the set call was successful by here.
                self.serialized_sample.get().unwrap().as_ref()
            })
        }
    }

    /// Get the kind associated with this serdata.
    pub fn kind(&self) -> Kind {
        match &self.sample.as_ref() {
            SampleOrKey::Sample { .. } => Kind::Data,
            SampleOrKey::Key { .. } => Kind::Key,
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
        let mut sertype = Box::new(Sertype::new(&type_name, topic_has_key));
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
