//! The [`Sertype`] represents the extension point to thread through
//! language-binding specific behavior through Cyclone.
//!
//! The name of the corresponding type, whether the associated topic is keyed,
//! and the set of interface functions are registered with Cyclone through the
//! [`cyclonedds_sys::ddsi_sertype`].

use crate::internal::ffi;

/// How samples of a topic are represented when Cyclone DDS hands them across the
/// FFI boundary as a bare `void *`.
///
/// This is the "application representation" in Cyclone DDS terms. Which one is in
/// use is fixed when the [`Sertype`] is created and read back out of it by the
/// serdata and sertype ops, since they are the ones that have to interpret the
/// pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    /// An [`InternalSample<T>`][crate::internal::ffi::InternalSample], the
    /// binding's own enum. Handles borrowed and owned samples and keys, but is
    /// meaningless to any other process.
    Internal,
    /// A bare `T`, laid out exactly as a C compiler would lay out the equivalent
    /// IDL type.
    ///
    /// Only used for [`FixedSizeTopic`][crate::FixedSizeTopic] types on a topic
    /// that opted into zero-copy, and only then is it sound for Cyclone DDS to
    /// `memcpy` samples into memory shared with other processes.
    Raw,
}

/// The extension point for wrapping [`cyclonedds_sys::ddsi_sertype`].
///
/// The embedded [`cyclonedds_sys::ddsi_sertype`] must stay the first field:
/// Cyclone DDS is handed a pointer to it and the ops cast that pointer back to a
/// `Sertype<T>`.
#[repr(C)]
#[derive(Debug)]
pub struct Sertype<T>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::ddsi_sertype,
    /// How Cyclone DDS passes samples of this topic across the FFI boundary.
    ///
    /// Kept here rather than derived from `T` so that enabling zero-copy cannot
    /// be done from safe code: it is set only through
    /// [`Sertype::new_raw`], whose caller has to satisfy
    /// [`FixedSizeTopic`][crate::FixedSizeTopic].
    pub(crate) representation: Representation,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Sertype<T>
where
    T: crate::Topicable,
{
    // NOTE: the struct update syntax has been used intentionally even though all
    // the fields are specified. This is to ensure that this code will still compile
    // even if the C library adds new ops in the future.
    #[allow(clippy::needless_update)]
    pub(crate) const SERTYPE_OPS: cyclonedds_sys::ddsi_sertype_ops =
        cyclonedds_sys::ddsi_sertype_ops {
            version: ffi::sertype_ops::SertypeVersion::V0.as_ffi(),
            arg: ffi::sertype_ops::ARG,
            free: Some(ffi::sertype_ops::free::<T>),
            zero_samples: Some(ffi::sertype_ops::zero_samples::<T>),
            realloc_samples: Some(ffi::sertype_ops::realloc_samples::<T>),
            free_samples: Some(ffi::sertype_ops::free_samples::<T>),
            equal: Some(ffi::sertype_ops::equal::<T>),
            hash: Some(ffi::sertype_ops::hash::<T>),

            // TODO integrate typelib feature?
            // type_id: Some(ffi::sertype_ops::type_id::<T>),
            // type_map: Some(ffi::sertype_ops::type_map::<T>),
            // type_info: Some(ffi::sertype_ops::type_info::<T>),
            // derive_sertype: Some(ffi::sertype_ops::derive_sertype::<T>),
            type_id: None,
            type_map: None,
            type_info: None,
            derive_sertype: None,

            get_serialized_size: Some(ffi::sertype_ops::get_serialized_size::<T>),
            serialize_into: Some(ffi::sertype_ops::serialize_into::<T>),
            ..ffi::sertype_ops::zeroed_sertype_ops()
        };
    // NOTE: the struct update syntax has been used intentionally even though all
    // the fields are specified. This is to ensure that this code will still compile
    // even if the C library adds new ops in the future.
    #[allow(clippy::needless_update)]
    pub(crate) const SERDATA_OPS: cyclonedds_sys::ddsi_serdata_ops =
        cyclonedds_sys::ddsi_serdata_ops {
            eqkey: Some(ffi::serdata_ops::eqkey::<T>),
            get_size: Some(ffi::serdata_ops::get_size::<T>),
            from_ser: Some(ffi::serdata_ops::from_ser::<T>),
            from_ser_iov: Some(ffi::serdata_ops::from_ser_iov::<T>),
            from_keyhash: Some(ffi::serdata_ops::from_keyhash::<T>),
            from_sample: Some(ffi::serdata_ops::from_sample::<T>),
            to_ser: Some(ffi::serdata_ops::to_ser::<T>),
            to_ser_ref: Some(ffi::serdata_ops::to_ser_ref::<T>),
            to_ser_unref: Some(ffi::serdata_ops::to_ser_unref::<T>),
            to_sample: Some(ffi::serdata_ops::to_sample::<T>),
            to_untyped: Some(ffi::serdata_ops::to_untyped::<T>),
            untyped_to_sample: Some(ffi::serdata_ops::untyped_to_sample::<T>),
            free: Some(ffi::serdata_ops::free::<T>),
            print: Some(ffi::serdata_ops::print::<T>),
            get_keyhash: Some(ffi::serdata_ops::get_keyhash::<T>),
            from_loaned_sample: Some(ffi::serdata_ops::from_loaned_sample::<T>),
            from_psmx: Some(ffi::serdata_ops::from_psmx::<T>),
            ..ffi::serdata_ops::zeroed_serdata_ops()
        };

    /// Create a new [`Sertype<T>`] using the binding's own sample
    /// representation.
    pub fn new(type_name: &std::ffi::CStr, topic_has_key: bool) -> Self {
        let inner = ffi::ddsi_sertype_new(
            type_name,
            &Self::SERTYPE_OPS,
            &Self::SERDATA_OPS,
            topic_has_key,
        );

        Sertype {
            inner,
            representation: Representation::Internal,
            phantom: std::marker::PhantomData,
        }
    }

    /// Create a new [`Sertype<T>`] whose samples are represented as a bare `T`,
    /// telling Cyclone DDS it may move them around as raw memory.
    ///
    /// ## Safety
    /// `T` must satisfy the [`FixedSizeTopic`][crate::FixedSizeTopic] contract.
    /// Cyclone DDS will `memcpy` `size_of::<T>()` bytes of a sample into memory
    /// that other processes read, and will present memory written by other
    /// processes to be read as a `T`.
    #[allow(
        unsafe_code,
        reason = "the memcpy-safe representation cannot be validated by the compiler"
    )]
    pub unsafe fn new_raw(type_name: &std::ffi::CStr, topic_has_key: bool) -> Self {
        // SAFETY: guaranteed by the caller of this function.
        let inner = unsafe {
            ffi::ddsi_sertype_new_memcpy_safe(
                type_name,
                &Self::SERTYPE_OPS,
                &Self::SERDATA_OPS,
                topic_has_key,
                std::mem::size_of::<T>(),
            )
        };

        Sertype {
            inner,
            representation: Representation::Raw,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for Sertype<T>
where
    T: crate::Topicable,
{
    fn drop(&mut self) {
        ffi::ddsi_sertype_fini(&mut self.inner);
    }
}
