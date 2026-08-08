//! The [`Sertype`] represents the extension point to thread through
//! language-binding specific behavior through Cyclone.
//!
//! The name of the corresponding type, whether the associated topic is keyed,
//! and the set of interface functions are registered with Cyclone through the
//! [`cyclonedds_sys::ddsi_sertype`].

use crate::internal::ffi;
use crate::internal::layout::Layout;

/// The extension point for wrapping [`cyclonedds_sys::ddsi_sertype`].
///
/// The embedded [`cyclonedds_sys::ddsi_sertype`] must stay the first field:
/// Cyclone DDS is handed a pointer to it and the ops cast that pointer back to a
/// `Sertype<T, R>`.
///
/// `R` is the topic's [`Representation`][crate::representation], which decides
/// how the ops interpret the sample pointers Cyclone DDS passes them. It is a type
/// parameter rather than a field so that each op is compiled for one
/// representation and the raw paths are unreachable for a type that cannot
/// support them.
#[repr(C)]
#[derive(Debug)]
pub struct Sertype<T, R>
where
    T: crate::Topicable,
{
    pub(crate) inner: cyclonedds_sys::ddsi_sertype,
    phantom: std::marker::PhantomData<(T, R)>,
}

impl<T, R> Sertype<T, R>
where
    T: crate::Topicable,
    R: Layout<T>,
{
    // NOTE: the struct update syntax has been used intentionally even though all
    // the fields are specified. This is to ensure that this code will still compile
    // even if the C library adds new ops in the future.
    #[allow(clippy::needless_update)]
    pub(crate) const SERTYPE_OPS: cyclonedds_sys::ddsi_sertype_ops =
        cyclonedds_sys::ddsi_sertype_ops {
            version: ffi::sertype_ops::SertypeVersion::V0.as_ffi(),
            arg: ffi::sertype_ops::ARG,
            free: Some(ffi::sertype_ops::free::<T, R>),
            zero_samples: Some(ffi::sertype_ops::zero_samples::<T, R>),
            realloc_samples: Some(ffi::sertype_ops::realloc_samples::<T, R>),
            free_samples: Some(ffi::sertype_ops::free_samples::<T, R>),
            equal: Some(ffi::sertype_ops::equal::<T, R>),
            hash: Some(ffi::sertype_ops::hash::<T, R>),

            // TODO integrate typelib feature?
            // type_id: Some(ffi::sertype_ops::type_id::<T>),
            // type_map: Some(ffi::sertype_ops::type_map::<T>),
            // type_info: Some(ffi::sertype_ops::type_info::<T>),
            // derive_sertype: Some(ffi::sertype_ops::derive_sertype::<T>),
            type_id: None,
            type_map: None,
            type_info: None,
            derive_sertype: None,

            get_serialized_size: Some(ffi::sertype_ops::get_serialized_size::<T, R>),
            serialize_into: Some(ffi::sertype_ops::serialize_into::<T, R>),
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
            from_ser: Some(ffi::serdata_ops::from_ser::<T, R>),
            from_ser_iov: Some(ffi::serdata_ops::from_ser_iov::<T, R>),
            from_keyhash: Some(ffi::serdata_ops::from_keyhash::<T, R>),
            from_sample: Some(ffi::serdata_ops::from_sample::<T, R>),
            to_ser: Some(ffi::serdata_ops::to_ser::<T>),
            to_ser_ref: Some(ffi::serdata_ops::to_ser_ref::<T>),
            to_ser_unref: Some(ffi::serdata_ops::to_ser_unref::<T>),
            to_sample: Some(ffi::serdata_ops::to_sample::<T, R>),
            to_untyped: Some(ffi::serdata_ops::to_untyped::<T, R>),
            untyped_to_sample: Some(ffi::serdata_ops::untyped_to_sample::<T, R>),
            free: Some(ffi::serdata_ops::free::<T>),
            print: Some(ffi::serdata_ops::print::<T>),
            get_keyhash: Some(ffi::serdata_ops::get_keyhash::<T>),
            from_loaned_sample: Some(ffi::serdata_ops::from_loaned_sample::<T, R>),
            from_psmx: Some(ffi::serdata_ops::from_psmx::<T, R>),
            ..ffi::serdata_ops::zeroed_serdata_ops()
        };

    /// Create a new [`Sertype<T, R>`].
    ///
    /// When `R` reports itself `memcpy`-safe, Cyclone DDS is told it may move
    /// samples of this topic as raw memory — which is what lets it hand them
    /// straight to a PSMX Instance. That is only reachable through
    /// [`ZeroCopy`][crate::representation::ZeroCopy], whose [`Layout`]
    /// implementation requires `T` to satisfy the
    /// [`FixedSizeTopic`][crate::FixedSizeTopic] contract, so the claim is
    /// discharged by the bound rather than by the caller.
    #[allow(
        unsafe_code,
        reason = "declaring the type memcpy-safe, which `R: Layout<T>` is what vouches for"
    )]
    pub fn new(type_name: &std::ffi::CStr, topic_has_key: bool) -> Self {
        let inner = if R::IS_MEMCPY_SAFE {
            // SAFETY: `R::IS_MEMCPY_SAFE` is only set by a `Layout<T>`
            // implementation that requires `T: FixedSizeTopic`, which is exactly
            // the contract this asks for.
            unsafe {
                ffi::ddsi_sertype_new_memcpy_safe(
                    type_name,
                    &Self::SERTYPE_OPS,
                    &Self::SERDATA_OPS,
                    topic_has_key,
                    std::mem::size_of::<T>(),
                )
            }
        } else {
            ffi::ddsi_sertype_new(
                type_name,
                &Self::SERTYPE_OPS,
                &Self::SERDATA_OPS,
                topic_has_key,
            )
        };

        Sertype {
            inner,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T, R> Drop for Sertype<T, R>
where
    T: crate::Topicable,
{
    fn drop(&mut self) {
        ffi::ddsi_sertype_fini(&mut self.inner);
    }
}
